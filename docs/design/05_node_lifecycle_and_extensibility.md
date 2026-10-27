# 05 - دورة حياة النود وقابلية التوسع (Node Lifecycle & Extensibility)

يوضح هذا المستند كيفية برمجة أنواع جديدة من العقد وتوسيع المحرر مستقبلاً عبر واجهات برمجية نظيفة، بالإضافة إلى إدارة الأداء والمعاينات المصغرة (Mini-Viewports).

---

## 1. واجهة العقدة الموسعة (`EditorNodeDefinition`)

لكي يدعم المحرر أي نوع من أنواع العقد (Mesh, Material, Light, Physics, Audio...)، نحدد Trait يفصل البيانات والواجهة:

```rust
pub trait EditorNodeDefinition: Send + Sync {
    /// الاسم المعروض للنود
    fn title(&self) -> &str;

    /// الفئة في قائمة الإضافة (مثلاً: "Geometry", "Shading", "Physics")
    fn category(&self) -> &str;

    /// المنافذ التي يطلبها النود عند إنشائه
    fn initial_pins(&self) -> (Vec<PinDefinition>, Vec<PinDefinition>);

    /// المعاينة المصغرة داخل البطاقة (Card View)
    fn render_card_content(&self, ui: &mut dyn MiniViewRenderer);

    /// واجهة الأدوات المتخصصة التي تظهر فقط عند الغوص داخل النود (Focus Mode)
    fn render_focus_workspace(&mut self, tools: &mut dyn FocusWorkspaceContext);
}
```

---

## 2. سجل العقد وقائمة الإضافة السريعة (Node Registry & Palette)

يوفر المحرر سجلاً مركزياً (`NodeRegistry`) يتيح للمطورين تسجيل أنواع نودات جديدة بسهولة:

```rust
#[derive(Default)]
pub struct NodeRegistry {
    factories: HashMap<String, Box<dyn Fn() -> Box<dyn EditorNodeDefinition>>>,
}
```

### قائمة الإضافة الذكية (Searchable Node Palette):
* عند الضغط على `Space` أو النقر بالزر الأيمن في الـ Canvas:
  * تظهر نافذة بحث منبثقة فورية.
  * كتابة أحرف سريعة (Fuzzy Search) مثل `"mesh"` أو `"mat"`.
  * عند اختيار النود، يتم إنشاؤه في إحداثيات موقع مؤشر الماوس داخل مساحة الـ Canvas، وتسجيل العملية في نظام الـ `Undo`.

---

## 3. معمارية المعاينة المصغرة وإدارة الأداء (Mini-Viewports & GPU Optimization)

عرض معاينة 3D حية داخل كل نود قد يرهق كارت الشاشة (GPU) إذا كان هناك عشرات النودات تعمل في نفس الوقت.

### استراتيجيات ترشيد الأداء:
1. **تحديث حسب الحاجة فقط (Dirty-Flag Updates):**
   * لا تعيد الكاميرا رسم المعاينة في كل فريم (60 FPS).
   * يتم التحديث **فقط** إذا تغيرت بيانات المدخلات أو قام المستخدم بتحريك الكائن.
2. **تخفيض التردد للنودات البعيدة (Throttled Frame-rate):**
   * النودات التي لا يراها المستخدم في الشاشة (خارج حدود الرؤية بفعل الـ Pan) يتم إيقاف كاميراتها تماماً (Frustum Culling).
   * النودات الصغيرة يتم رسمها بتردد منخفض (مثلاً 15 FPS).
3. **أهداف الرسوميات المخصصة (Render Targets):**
   * في Bevy:
     ```rust
     Camera3d {
         target: RenderTarget::Image(node_preview_image_handle),
         ..default()
     }
     ```
   * تقوم `bevy_luma` بعرض هذه الصورة كعنصر رسومي ناعم الحواف (SDF Rounded Quad) داخل بطاقة النود.

---

## 4. التحول إلى وضع ملء الشاشة (Focus Viewport Switch)

عند الغوص داخل النود (Dive-In):
* يتم **إيقاف** الكاميرا المصغرة (Off-screen rendering) لتوفير موارد الـ GPU.
* تتولى **الكاميرا الرئيسية للمحرر (Main 3D Viewport Camera)** رسم المشهد الخاص بهذا النود بأعلى دقة ممكنة وإطارات كاملة (Full Native Resolution).
* ترتبط أدوات النود بالمدخلات المباشرة (Mouse Gizmo Dragging, Keyboard Shortcuts).
* عند الخروج (Exit Focus)، تعود الكاميرا الرئيسية لعرض مساحة الـ Canvas، وتعود الكاميرا المصغرة للعمل الخفيف في الخلفية.
