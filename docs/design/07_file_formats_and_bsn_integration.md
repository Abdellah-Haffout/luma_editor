# 07 - صيغ الملفات والتكامل مع BSN (File Formats & BSN Integration)

يشرح هذا المستند استراتيجية حفظ وتصدير المشاهد في `Luma Editor`، والعلاقة بين ملف المشروع المصدر الخاص بالمحرر وصيغة المشاهد الرسمية لمحرك Bevy المسماة **`bsn` (Bevy Scene Notation)**.

---

## 1. الفلسفة الأساسية: الفصل بين المصدر والمخرجات (Source vs Runtime)

في المحركات والبرامج الإبداعية الاحترافية (مثل Blender وSubstance Designer وUnreal)، هناك فصل صارم بين:
* **حالة التأليف في المحرر (Authoring / Source State):** ما يحتاجه المطور لمواصلة العمل وتعديل الـ Graph بصرياً.
* **حالة التشغيل في اللعبة (Runtime / Baked State):** ما يحتاجه محرك اللعبة للتشغيل بأعلى أداء ممكن دون أي استهلاك زائد للذاكرة.

```
┌────────────────────────────────────────────────────────────────────────┐
│                          Luma Editor (Workplace)                       │
│                                                                        │
│   ┌──────────────┐       ┌──────────────┐       ┌─────────────┐        │
│   │ AssetLoader  │──────▶│  Mesh Node   │──────▶│ Entity Node │        │
│   └──────────────┘       └──────────────┘       └─────────────┘        │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                         [Auto-Bake on Save (Ctrl+S)]
                                    │
             ┌──────────────────────┴──────────────────────┐
             ▼                                             ▼
   ┌────────────────────┐                        ┌────────────────────┐
   │    scenes/main     │                        │    scenes/main     │
   │      .luma         │                        │       .bsn         │
   │   (ملف المصدر)     │                        │   (ملف اللعبة)     │
   ├────────────────────┤                        ├────────────────────┤
   │ - مواقع النودات     │                        │ - كيانات Bevy      │
   │ - الأسلاك والصلات  │                        │ - المكونات Components│
   │ - Zoom و Pan       │                        │ - مراجع الأصول     │
   │ - معطيات المحرر    │                        │ - صفر بيانات محرر! │
   └────────────────────┘                        └────────────────────┘
             │                                             │
      يفتحه المحرر للتعديل                             تقرأه اللعبة مباشرة
```

---

## 2. ملف المصدر الخاص بالمحرر (`.luma` - RON Schema)

يُحفظ ملف المحرر بصيغة **RON (Rusty Object Notation)** لكونه:
1. اللغة الطبيعية لمنظومة Rust وBevy.
2. نصي وواضح وسهل القراءة والمراجعة في نظام تتبع النسخ (Git Diffs & Merges).
3. يدعم الأنواع المعقدة والـ Enums بأمان تام.

### الهيكل القياسي لملف الـ Graph:

```ron
LumaGraph(
    version: 1,
    meta: (
        author: "Abdellah",
        created_at: 1773446400,
        editor_version: "0.1.0",
    ),
    canvas: (
        pan: (120.0, -85.5),
        zoom: 1.15,
    ),
    nodes: [
        (
            id: 1,
            kind: "MeshNode",
            title: "Player Mesh",
            position: (250.0, 100.0),
            size: (220.0, 160.0),
            is_collapsed: false,
            // بيانات مخصصة بحسب نوع النود
            payload: MeshPayload(
                mesh_type: AssetPath("models/character.glb#Mesh0"),
                subdivisions: 1,
            ),
        ),
        (
            id: 2,
            kind: "MaterialNode",
            title: "Armor Material",
            position: (250.0, 320.0),
            size: (200.0, 140.0),
            payload: MaterialPayload(
                base_color: (0.8, 0.1, 0.1, 1.0),
                roughness: 0.35,
                metallic: 0.8,
            ),
        ),
        (
            id: 3,
            kind: "EntityAssembler",
            title: "Player Entity",
            position: (600.0, 200.0),
            size: (240.0, 180.0),
            payload: EntityPayload(
                name: "Hero",
                transform: (
                    translation: (0.0, 1.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                ),
            ),
        ),
    ],
    connections: [
        (
            id: 101,
            from: (node: 1, pin: 0),
            to: (node: 3, pin: 0),
        ),
        (
            id: 102,
            from: (node: 2, pin: 0),
            to: (node: 3, pin: 1),
        ),
    ],
)
```

---

## 3. محرك الترجمة والتصدير (The Graph Compiler / Baker)

عند حفظ المشروع أو عند طلب تصديره:
1. **الفرز الطوبولوجي (Topological Sort):** يقوم المحرر بترتيب العقد في الـ DAG لضمان توليد البيانات بالترتيب الصحيح (من المدخلات إلى مخرجات المشهد).
2. **توليد الكيانات والمكونات (Entity & Component Assembly):**
   * تجمع عقدة `EntityAssembler` المكونات القادمة من الـ `MeshNode` والـ `MaterialNode` والـ `PhysicsNode`.
3. **الكتابة إلى ملف `bsn` نقي:**
   يتم توليد ملف المشهد القياسي لـ Bevy:

```ron
// scenes/player.bsn
(
    entities: {
        0: (
            components: {
                "bevy_core::name::Name": "Hero",
                "bevy_transform::components::transform::Transform": (
                    translation: (0.0, 1.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    scale: (1.0, 1.0, 1.0),
                ),
                "bevy_pbr::pbr_material::MeshMaterial3d": (
                    base_color: (0.8, 0.1, 0.1, 1.0),
                    perceptual_roughness: 0.35,
                    metallic: 0.8,
                ),
                "bevy_render::mesh::Mesh3d": "models/character.glb#Mesh0",
            },
        ),
    },
)
```

---

## 4. تجربة المستخدم: المزامنة التلقائية (Auto-Bake on Save)

لتوفير أقصى قدر من الراحة والسرعة للمطور:
1. يضغط المطور `Ctrl + S` في محرر Luma.
2. المحرر يحفظ `player.luma` ويولد فوراً وبشكل غير مرئي `player.bsn`.
3. إذا كانت لعبة Bevy تعمل في الخلفية بنظام **Asset Hot-Reloading**، فسيتم تحديث المشهد في اللعبة في نفس الثانية دون إعادة تشغيلها!

### في كود لعبة Bevy:
المطور لا يحتاج لأي كود من المحرر؛ بل يكتب فقط الكود المعتاد في Bevy:
```rust
fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicSceneRoot(asset_server.load("scenes/player.bsn")));
}
```

---

## 5. إدارة الإصدارات والترقية (Schema Migrations)

لكل ملف `.luma` حقل `version: u32`.  
عند تحديث المحرر مستقبلاً وتغيير بنية بعض النودات:
* يمتلك المحرر محرك ترقية تلقائي (`GraphMigrator`).
* إذا فُتح ملف برقم إصدار أقدم (مثلاً `version: 1` والمحرر في `version: 2`):
  * يطبق المحرر دوال الترقية (`v1_to_v2`) تلقائياً في الذاكرة.
  * يحفظ الملف بالنسخة الجديدة عند أول حفظ، مما يمنع تعطل مشاريع المستخدمين القديمة.
