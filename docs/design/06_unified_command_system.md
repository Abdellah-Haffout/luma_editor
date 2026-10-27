# 06 - نظام الأوامر الموحد (Unified Command Architecture)

يعالج هذا المستند المعمارية الشاملة لنظام الأوامر (Command System) في `Luma Editor`.  
بدلاً من حصر الأوامر في مجرد مكدس للتراجع (Undo/Redo)، يعمل هذا النظام كـ **جهاز عصبي مركزي (Central Nervous System)** يُشغل كافة جوانب المحرر: لوحة الأوامر السريعة (Command Palette)، الاختصارات القابلة للتخصيص، تسجيل الماكرو، الحماية من الانهيار، والتكامل مع الذكاء الاصطناعي والسكريبتات.

---

## 1. مخطط تدفق الأوامر المركزي (The Central Pipeline)

كل حدث أو تفاعل في المحرر يمر عبر خط أنابيب موحد ومحكم:

```
    ┌──────────────────────┐   ┌──────────────────────┐   ┌──────────────────────┐
    │ Keyboard / Shortcuts │   │   Command Palette    │   │  AI Agent / Scripts  │
    └──────────┬───────────┘   └──────────┬───────────┘   └──────────┬───────────┘
               │                          │                          │
               └──────────────────────────┼──────────────────────────┘
                                          ▼
                             ┌─────────────────────────┐
                             │    Command Dispatcher   │
                             └────────────┬────────────┘
                                          │
       ┌──────────────────┬───────────────┴───────────────┬──────────────────┐
       ▼                  ▼                               ▼                  ▼
┌──────────────┐   ┌──────────────┐               ┌──────────────┐   ┌──────────────┐
│  Validation  │   │  Execution   │               │ Undo Engine  │   │ Crash Log    │
│ (Context OK?)│   │ (Graph/World)│               │ (UndoDelta)  │   │(session.log) │
└──────────────┘   └──────────────┘               └──────────────┘   └──────────────┘
```

---

## 2. الهياكل الأساسية لنظام الأوامر

### أ) معرّف ووصف الأمر (`CommandDescriptor`)
يمتلك كل أمر هوية واضحة وبيانات وصفية تمكّن واجهات المستخدم من عرضه والبحث عنه:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

#[derive(Debug, Clone)]
pub struct CommandDescriptor {
    pub id: CommandId,
    pub title: String,          // الاسم المعروض: "Delete Selected Nodes"
    pub category: String,       // التصنيف: "Graph", "View", "Tools"
    pub description: String,    // شرح مختصر
    pub default_shortcut: Option<String>, // مثال: "Ctrl+X" أو "Delete"
    pub is_undoable: bool,      // هل يؤثر على البيانات ويتطلب Undo؟
}
```

### ب) واجهة الأمر التنفيذية (`EditorCommand`)
```rust
pub trait EditorCommand: Send + Sync {
    fn descriptor(&self) -> &CommandDescriptor;

    /// فحص السياق: هل يمكن تنفيذ هذا الأمر حالياً؟
    /// مثال: أمر "Delete" لا يمكن تنفيذه إذا لم يكن هناك نود محدد.
    fn can_execute(&self, ctx: &EditorContext) -> bool { true }

    /// التنفيذ الفعلي للأمر
    fn execute(
        &mut self, 
        ctx: &mut EditorContext, 
        args: &CommandArgs
    ) -> Result<CommandOutput, CommandError>;
}
```

---

## 3. الأنظمة الفرعية المشتقة من نظام الأوامر

### 1. لوحة الأوامر السريعة (Command Palette - `Ctrl + P` أو `Space`)
* تتيح للمستخدم البحث في كافة أوامر المحرر فورياً (Fuzzy Search).
* لا حاجة لبرمجة كود خاص لكل زر في القوائم؛ فبمجرد تسجيل الأمر في `CommandRegistry`، يظهر تلقائياً في نافذة البحث.
* ترشيح الأوامر حسب السياق: إذا كان المستخدم في وضع الـ `Dive-In` داخل `MeshNode`، تظهر أوامر تعديل المجسمات في المقدمة وتختفي أوامر الـ Canvas غير المناسبة.

### 2. محرك الاختصارات الديناميكي (Dynamic Keybinding Manager)
* يتم فصل منطق الكيبورد تماماً عن منطق الأوامر.
* يتم تخزين الاختصارات في ملف إعدادات مرن (مثلاً `keybindings.ron`):
  ```ron
  {
      "node.delete": "Delete",
      "canvas.zoom_fit": "F",
      "node.dive_in": "Tab",
      "node.spawn_palette": "Space",
  }
  ```
* يمكن للمطور إعادة تخصيص أي اختصار من نافذة الإعدادات بسهولة ودون إعادة تجميع الكود (No Recompilation).

### 3. الحماية واستعادة الجلسة عند الانهيار (Crash Recovery / Event Sourcing)
* يسجل الـ `CommandDispatcher` كل أمر قابل للتنفيذ في ملف سجل تتابعي خفيف (`session_journal.bin`).
* في حال إغلاق البرنامج فجأة أو انقطاع الكهرباء:
  * عند الإقلاع التالي، يكتشف المحرر وجود جلسة لم تُحفظ بشكل طبيعي.
  * يقوم المحرر بقراءة السجل وإعادة تنفيذ الأوامر بالترتيب (**Command Replay**) لاسترجاع المشهد بدقة تامة.

### 4. نظام الماكرو والأتمتة (Macro Recording)
* يستطيع المستخدم تفعيل وضع "تسجيل ماكرو" (Record Macro).
* يجمع المحرر الأوامر التي ينفذها المطور (مثلاً: إضافة نودين، ضبط إحداثيات، وتوصيل منفذ).
* تُحفظ هذه السلسلة باسم ماكرو جديد يُسجل كـ `Command` جديد كلياً في الـ Registry يمكن تشغيله بضغطة زر واحدة!

### 5. واجهة برمجية جاهزة للذكاء الاصطناعي (AI Agent & Automation API)
* عندما نريد ربط مساعد ذكاء اصطناعي (AI Copilot) مستقبلاً، لن يحتاج للتعامل مع رسوميات الـ UI.
* يستطيع الـ AI إرسال أوامر قياسية بصيغة JSON أو نصوص عبر قناة أوامر موحدة:
  ```json
  {
      "command": "graph.connect_pins",
      "args": {
          "source_node": 102,
          "source_pin": 1,
          "target_node": 105,
          "target_pin": 0
      }
  }
  ```
  يقوم المحرر بفحص السياق، التحقق، التنفيذ، وتسجيل الـ Undo، تماماً كما لو قام مستخدم بشري بالسحب والتوصيل!

---

## 4. التكامل مع مكدس التراجع (`Undo/Redo`)

ليس كل أمر يتطلب حفظه في الـ Undo:
* **أوامر العرض والحركة (Ephemeral Commands):** مثل `canvas.pan`, `canvas.zoom_to_fit`, `editor.toggle_fullscreen`. هذه الأوامر تنفذ وتحدث الشاشة لكن لا تضاف لمكدس الـ Undo.
* **أوامر تعديل البيانات (Mutating Commands):** مثل `node.create`, `wire.disconnect`, `node.move`. هذه الأوامر تولد تلقائياً `UndoDelta` ويتم دفعها لمكدس التراجع (`history_stack.push(delta)`).

---

## 5. خلاصة المعمارية

تحويل كل فعل في المحرر إلى **Command** من اليوم الأول يمنح `Luma Editor`:
1. قابلية صيانة واختبار لا مثيل لها (Headless Testing للأوامر بدون فتح نافذة).
2. واجهة مستخدم احترافية وحديثة تدعم البحث السريع (Command Palette).
3. مرونة كاملة في الاختصارات والماكرو والعمل التعاوني ومستقبل الذكاء الاصطناعي.
