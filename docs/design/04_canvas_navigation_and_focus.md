# 04 - فضاء العمل والتنقل والغوص (Canvas Navigation & Focus)

يشرح هذا المستند الرياضيات المسؤولة عن تحويل الإحداثيات (Pan / Zoom)، إدارة التحديد، والآلية المحورية للمحرر: **الغوص داخل العقدة (Dive-In / Focus Mode)**.

---

## 1. فضاءات الإحداثيات والتحويل الرياضي (Coordinate Spaces)

في المحرر هناك فضاءان رئيسيان للإحداثيات:
1. **فضاء الشاشة (Screen Space):** إحداثيات البكسل على نافذة التطبيق من `(0, 0)` إلى `(WindowWidth, WindowHeight)`.
2. **فضاء مساحة الـ Graph (Canvas Space):** فضاء افتراضي لا نهائي حيث تتواجد النودات والأسلاك.

```rust
#[derive(Debug, Clone, Copy)]
pub struct CanvasTransform {
    pub pan: Vec2,   // الإزاحة الحالية
    pub zoom: f32,   // معامل التكبير (الافتراضي 1.0، المقبول مثلاً 0.1 إلى 3.0)
}

impl CanvasTransform {
    /// تحويل نقطة من الشاشة إلى مساحة الـ Canvas
    pub fn screen_to_canvas(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos - self.pan) / self.zoom
    }

    /// تحويل نقطة من مساحة الـ Canvas إلى الشاشة
    pub fn canvas_to_screen(&self, canvas_pos: Vec2) -> Vec2 {
        (canvas_pos * self.zoom) + self.pan
    }
}
```

### التكبير المتمركز حول المؤشر (Zoom to Cursor):
عند استخدام عجلة الماوس للتكبير/التصغير، يجب ألا تقفز النودات؛ بل يتم تعديل الـ `pan` لتبقى النقطة التي تحت المؤشر ثابتة تماماً في مساحة الـ Canvas:
$$\text{pan}_{\text{new}} = \text{mouse\_pos} - (\text{mouse\_pos} - \text{pan}_{\text{old}}) \times \frac{\text{zoom}_{\text{new}}}{\text{zoom}_{\text{old}}}$$

---

## 2. نظام التحديد المتعدد (Selection System)

```rust
use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
pub struct SelectionState {
    pub selected_nodes: HashSet<NodeId>,
    pub selected_connections: HashSet<ConnectionId>,
    pub box_selection: Option<Rect>, // صندوق التحديد الجاري سحبه بالماوس
}
```

* **التحديد المفرد:** النقر المباشر على نود أو سلك.
* **إضافة/إزالة بالتحديد:** الضغط على `Shift` أثناء النقر.
* **صندوق التحديد (Marquee Box Selection):** سحب الماوس على خلفية فارغة؛ يتم تحويل حدود الصندوق إلى `Canvas Space`، ويتم تحديد جميع النودات التي تتقاطع أغلفتها المحيطة (Bounding Boxes) معه.

---

## 3. آلية الغوص والتركيز (The Dive-In / Focus System)

هذه هي الميزة الجوهرية لـ `Luma Editor` التي تفصله عن بقية المحررات.

```
┌────────────────────────────────────────────────────────┐
│ [Breadcrumb Header]: Root Graph  >  Mesh_Player  [❌]  │
├────────────────────────────────────────────────────────┤
│                                                        │
│                                                        │
│                    Focused Workspace                   │
│              (Full Viewport for Mesh Node)             │
│                                                        │
│                                                        │
│ [Contextual Toolbar]:                                  │
│ [ Gizmo: Translate | Rotate | Scale ] [ Wireframe: ON ]│
└────────────────────────────────────────────────────────┘
```

### أ) مكدس مسار التنقل (Breadcrumb & Focus Stack)
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceContext {
    /// العرض العام لكل النودات
    RootGraph,
    
    /// داخل عقدة معينة للتركيز عليها وتحريرها بملء الشاشة
    FocusedNode {
        node_id: NodeId,
        node_kind: String,
    },
}

#[derive(Debug, Default)]
pub struct NavigationState {
    pub stack: Vec<WorkspaceContext>,
}
```

### ب) حالات عرض النود (Node Display States)
يمر النود بثلاث حالات بصرية:
1. **بطاقة النود المصغرة (Card Mode):**
   * يعرض في مساحة الـ Graph العادية.
   * يحتوي على منافذ، ومعاينة تفاعلية صغيرة (Mini-Viewport)، وزر التوسيع `[🔍 Focus / Dive-In]`.
2. **الانتقال السلس (Transition Mode):**
   * انيميشن تكبير سلس يمدد النود ليملأ الشاشة بالكامل (أو انتقال عدسة الكاميرا).
3. **مساحة العمل الكاملة (Focused Workspace Mode):**
   * يملأ الشاشة بنسبة 100%.
   * يتم استبدال قوائم المحرر العامة بـ **شريط أدوات مخصص فقط لهذا النود** (Contextual Toolbar).
   * يظهر في الأعلى شريط المسار (Breadcrumbs) مع زر خروج سريع `[Back to Graph]` أو بالضغط على زر `Esc`.

---

## 4. عزل الأدوات حسب التخصص (Contextual Tool Isolation)

عند تفعيل الـ `FocusedNode`:
* يستشير المحرر نوع النود (`node_kind`).
* إذا كان `MeshNode`: يتم تحميل أدوات معالجة المجسمات فقط (Translate/Rotate/Scale Gizmos، التناظر، تقسيم الأوجه).
* يتم إخفاء أي أدوات لا تنتمي لنطاق هذا النود، مما يمنح المطور **أقصى درجات التركيز وخلو الواجهة من التشتيت**.
