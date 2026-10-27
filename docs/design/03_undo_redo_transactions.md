# 03 - محرك التراجع والإعادة والمعاملات (Undo / Redo & Transactions)

يعد نظام الـ Undo/Redo العمود الفقري لاستقرار المحرر وثقة المستخدم. في بيئة الـ Node Graph، العمليات بطبيعتها مترابطة (حذف نود يستوجب حذف أسلاكه، ونقل نود قد يتبعه نقل نودات أخرى).

لذلك نعتمد معمارية **المعاملات العكسية (Reversible Command Pattern with Transactions)**.

---

## 1. واجهة الأوامر التجريدية (`EditCommand`)

كل إجراء يغير حالة الـ Graph يجب أن يطبق هذه الواجهة:

```rust
pub trait EditCommand: Send + Sync {
    /// تنفيذ الأمر وتطبيق التغيير على الـ Graph
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), CommandError>;

    /// التراجع عن الأمر وإعادة الـ Graph لحالته السابقة تماماً
    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), CommandError>;

    /// وصف نصي للعملية يظهر في شريط القوائم (مثلاً: "Move Nodes", "Connect Wire")
    fn description(&self) -> &str;
}
```

---

## 2. الأوامر الذرية الأساسية (Atomic Commands)

هذه هي اللبنات الأساسية التي تُبنى منها كل التعديلات:

### أ) إضافة وحذف النود (`AddNodeCommand` / `RemoveNodeCommand`)
```rust
pub struct AddNodeCommand {
    node: Option<Node>,
    created_id: Option<NodeId>,
}

pub struct RemoveNodeCommand {
    node_id: NodeId,
    // يتم حفظ نسخة من النود المحذوف لاستعادته عند الـ Undo
    cached_node: Option<Node>,
}
```

### ب) التوصيل والفصل (`ConnectCommand` / `DisconnectCommand`)
```rust
pub struct ConnectCommand {
    source_pin: PinId,
    target_pin: PinId,
    created_connection_id: Option<ConnectionId>,
}

pub struct DisconnectCommand {
    connection_id: ConnectionId,
    cached_connection: Option<Connection>,
}
```

### ج) تحريك مجموعة نودات (`MoveNodesCommand`)
```rust
pub struct MoveNodesCommand {
    pub moves: Vec<(NodeId, Vec2 /* old_pos */, Vec2 /* new_pos */)>,
}
```

---

## 3. المعاملات المركبة (Compound Transactions)

تعتبر المعاملات المركبة حاسمة للعمليات التي تؤثر على عدة عناصر معاً.

### سيناريو حذف نود مرتبط بعدة أسلاك:
إذا حذف المستخدم نوداً يمتلك 4 أسلاك متصلة:
1. يتم فتح معاملة: `history.begin_transaction("Delete Node")`.
2. يتم فصل الأسلاك الأربعة وتسجيل 4 أوامر `DisconnectCommand`.
3. يتم حذف النود وتسجيل أمر `RemoveNodeCommand`.
4. يتم إغلاق المعاملة: `history.end_transaction()`.

```rust
pub struct CompoundTransaction {
    description: String,
    commands: Vec<Box<dyn EditCommand>>,
}

impl EditCommand for CompoundTransaction {
    fn execute(&mut self, graph: &mut GraphStore) -> Result<(), CommandError> {
        for cmd in &mut self.commands {
            cmd.execute(graph)?;
        }
        Ok(())
    }

    fn undo(&mut self, graph: &mut GraphStore) -> Result<(), CommandError> {
        // التراجع يتم دائماً بالترتيب العكسي!
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(graph)?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}
```

> **النتيجة:** عندما يضغط المستخدم `Ctrl + Z`، يعود النود وتعود كل أسلاكه في **خطوة واحدة سلسة**.

---

## 4. دمج العمليات المستمرة (Continuous Action Merging / Debouncing)

أثناء تفاعل المستخدم:
* **سحب النودات بالماوس (Dragging Nodes):** تتغير إحداثيات النود في كل فريم (60 مرة بالثانية).
* **القاعدة:** لا نسجل أمراً في كل فريم!
  * عند بداية السحب (`Mouse Down`): نحفظ الموقع المبدئي (`start_pos`).
  * أثناء السحب: نحدث موقع النود في الـ Canvas للعرض المباشر.
  * عند الإفلات (`Mouse Up`): إذا كان هناك فارق في الموقع، نسجل أمراً واحداً فقط:
    `MoveNodesCommand { old_pos: start_pos, new_pos: current_pos }`.

---

## 5. مكدس التاريخ وإدارة الحفظ (`HistoryStack`)

```rust
pub struct HistoryStack {
    undo_stack: Vec<Box<dyn EditCommand>>,
    redo_stack: Vec<Box<dyn EditCommand>>,
    max_depth: usize, // مثلاً: 100 خطوة
    
    // مؤشر لموقع آخر عملية حفظ على القرص (Dirty Flag)
    saved_index: Option<usize>,
}
```

* **الحد الأقصى للذاكرة:** عند تجاوز `max_depth`، يتم إسقاط أقدم أمر تلقائياً لمنع تسريب الذاكرة.
* **إلغاء الـ Redo:** عند تنفيذ أي أمر جديد أثناء وجود أوامر في الـ Redo، يتم مسح مكدس الـ Redo بالكامل.
* **حالة التعديل (Is Dirty):** إذا كان مؤشر الـ Undo الحالي يختلف عن `saved_index`، تظهر علامة النجمة (`*`) بجانب اسم المشروع في شريط العنوان للدلالة على وجود تعديلات غير محفوظة.
