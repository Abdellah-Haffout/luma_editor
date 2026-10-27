# 08 - منظومة واجهة المستخدم والتكامل مع Bevy Luma (UI Stack & Luma Integration)

يعتمد محرر `Luma Editor` على منظومة واجهة مستخدم حديثة ومتكاملة تجمع بين ثلاث ركائز أساسية:
1. **`bevy_luma`:** محرك الواجهات الهجين في فضاء العالم (SDF-based World-Space UI).
2. **`bevy_luma_widgets`:** مكتبة المكونات الجاهزة وعالية المستوى المبنية بـ `bsn!` (Cards, Breadcrumbs, Modals, Sliders, Toasts...).
3. **`bevy_ui`:** نظام واجهة المستخدم الأصلي لمحرك Bevy لإدارة النوافذ والطبقات الشاشية (Screen-Space Root & Overlays).

---

## 1. توزيع الأدوار في واجهة المحرر (Role Distribution)

```
┌────────────────────────────────────────────────────────────────────────┐
│  bevy_ui (Root Screen Container & Window Shell)                        │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ bevy_luma_widgets::Breadcrumb ("Root Graph > Mesh_Hero")         │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │                                                                  │  │
│  │  bevy_luma (SDF Infinite Canvas Space - Pan & Zoom)             │  │
│  │                                                                  │  │
│  │    ┌───────────────────────────┐      (SDF Bézier Wire)          │  │
│  │    │ bevy_luma_widgets::Card   │──────────────────┐              │  │
│  │    │ [Node Header & Icon]      │                  ▼              │  │
│  │    │ [Mini-Viewport Target]    │        ┌───────────────────┐    │  │
│  │    │ [Pins & Sockets]          │        │ bevy_luma_widgets │    │  │
│  │    │ [Dive-In Button]          │        │      ::Card       │    │  │
│  │    └───────────────────────────┘        └───────────────────┘    │  │
│  │                                                                  │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │ bevy_luma_widgets::Modal (Command Palette: Ctrl+P / Space)       │  │
│  │  - Input (Search)                                                │  │
│  │  - Kbd (Shortcuts Badges)                                        │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │ bevy_luma_widgets::Toast ("Auto-baked to scenes/player.bsn")     │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. تفصيل استخدام كل مكتبة

### أ) دور `bevy_luma`: مساحة الـ Graph والأسلاك (SDF Canvas Engine)
* **المسؤولية:** رسم فضاء الـ Canvas اللانهائي ورسم الأسلاك والمنافذ بدقة متناهية.
* **الميزات المستخدمة:**
  * **SDF Rendering:** تتيح تكبير وتصغير الـ Canvas (Zoom) إلى أقصى حد بنعومة فكتورية تامة دون أي بكسلة.
  * **World-Space Transformations:** إمكانية تحريك الـ Canvas وإدارتها بسلاسة كعناصر في فضاء ثنائي/ثلاثي الأبعاد.
  * **منحنيات الأسلاك (SDF Splines):** رسم خطوط الاتصال بين المنافذ كأشكال SDF منحنية وناعمة مع إمكانية التوهج (Glow) عند مرور التيارات والبيانات.

---

### ب) دور `bevy_luma_widgets`: مكونات المحرر وعناصر التحكم
تمتلك `bevy_luma_widgets` ترسانة متكاملة من العناصر التي تناسب محرراً بهذا النمط بدقة:

1. **بطاقة النود (`Card`):**
   * هي الهيكل البصري لكل عقدة على الـ Canvas (حواف مستديرة ناعمة، تأثيرات الظل والارتفاع SDF Elevation، وفصل الهيدر عن المحتوى).
2. **مسار التنقل للـ Focus (`Breadcrumb`):**
   * يظهر في أعلى الشاشة عند الدخول في وضع الغوص (`Dive-In`):  
     `Scene Root > Player Mesh > [Back Button]` للتنقل الهرمي بين المستويات.
3. **لوحة الأوامر السريعة (`Modal` + `Input` + `Kbd`):**
   * نافذة منبثقة مع خلفية ضبابية (Backdrop) عند الضغط على `Ctrl + P` أو `Space`.
   * حقل إدخال (`Input`) للبحث الفوري، ومفاتيح كيبورد مجهزة (`Kbd`) تعرض الاختصارات بجانب الأوامر.
4. **عناصر التحكم بالخصائص (Contextual Controls):**
   * `Slider` للقيم الرقمية وأبعاد المجسمات.
   * `Toggle` و `Checkbox` للميزات المفعلة/المعطلة.
   * `ColorSwatch` لاختيار ألوان المواد والمؤشرات.
5. **الإشعارات السريعة (`Toast`):**
   * تنبيهات أنيقة في زاوية الشاشة عند نجاح الترجمة التلقائية للمشهد (`Auto-Baked to .bsn`) أو عند نسخ النودات.

---

### ج) دور `bevy_ui`: الحاوية الأصلية والطبقات الشاشية
* **المسؤولية:**
  * تشكيل الحاوية الشاشية العليا (Screen-Space Root).
  * تثبيت القوائم العلوية الدائمة (Main Menu Bar) وشريط الحالة السفلي (Status Bar).
  * توجيه أحداث الفأرة والنوافذ والتنسيق العام مع محرك Bevy.

---

## 3. التكامل التعبيري بواسطة `bsn!` (Bevy Scene Notation)

بفضل بنية `bevy_luma_widgets` المصممة لدعم ماكرو `bsn!`، يمكن بناء بطاقة النود بتعبير بصري أنيق ومباشر:

```rust
// مثال لبناء بطاقة نود بواسطة bsn! و bevy_luma_widgets
fn spawn_mesh_node_ui(theme: &Theme, title: &str) -> impl Scene + use<> {
    bsn! {
        Card {
            padding: USides::all(12.0),
            border_radius: 8.0,
            background_color: theme.colors.surface,
        }
        Children [
            // شريط العنوان
            (
                UNode::default()
                ULayout {
                    display: UDisplay::Flex,
                    justify_content: UJustifyContent::SpaceBetween,
                }
                Children [
                    (UText {
                        text: title.to_string(),
                        font_size: 14.0,
                        color: Color::WHITE,
                    }),
                    // زر التوسيع للـ Dive-In
                    (Button::ghost("🔍 Focus")),
                ]
            ),
            // مساحة المعاينة المصغرة (Mini-Viewport Container)
            (
                UNode {
                    width: UVal::Px(180.0),
                    height: UVal::Px(120.0),
                    background_color: Color::BLACK,
                }
            ),
        ]
    }
}
```

---

## 4. الميزة التنافسية لهذه المنظومة

* **أداء فائق:** بالاعتماد على محرك التقديم المبني لـ Bevy بدون طبقات وسيطة أو برمجيات قديمة.
* **انسجام بصري وتصميم موحد:** جميع مكونات المحرر تشارك نفس الـ `Theme` والـ Design System.
* **خلو كامل من التبعيات الخارجية الثقيلة:** لا حاجة للاعتماد على `egui` في واجهة المستخدم، مما يحافظ على نقاء المشروع وتكامله التام مع Bevy.
