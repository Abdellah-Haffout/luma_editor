# قائمة المهام وخارطة طريق تطوير Luma Editor (Implementation Checklist)

تغطي هذه القائمة كافة المهام البرمجية والهندسية لتنفيذ محرر `Luma Editor` خطوة بخطوة وفقاً لوثائق التصميم (من `00` إلى `08`).

---

## 🟢 المرحلة 1: النواة البرمجية ومحرك التاريخ (Core & History Engine)
> **المرجع:** `01_graph_data_model.md`، `02_connections_and_wires.md`، `03_undo_redo_transactions.md`

- [x] **1.1 المعرفات الفريدة القوية (Strong-Typed IDs)**
  - [x] تعريف `NodeId`, `PinId`, `ConnectionId` مع `Display` و `Serialize`.
  - [x] بناء `IdGenerator` آمن للخيوط (Atomic Counter) لتوليد المعرفات.
- [x] **1.2 نموذج بيانات الـ Graph الصافي (Core Data Model)**
  - [x] تعريف `Node`, `Pin`, `Connection`, `DataType`, `PinDirection`, `PinCapacity`.
  - [x] بناء حاوية `GraphStore` مع فهارس البحث السريع وعمليات الإضافة والحذف والفصل.
- [x] **1.3 محرك التحقق الصارم (Validation Engine)**
  - [x] منع التوصيل الذاتي (`SelfConnection`).
  - [x] التحقق من الاتجاه (Output إلى Input فقط).
  - [x] فحص تطابق أنواع البيانات (`DataType::is_compatible_with`).
  - [x] تطبيق قيود سعة المنافذ (`PinCapacity::Single` vs `Multiple`).
- [x] **1.4 خوارزمية كشف الحلقات (DAG Cycle Detection)**
  - [x] منع الدورات المغلقة باستخدام خوارزمية BFS/DFS لضمان رسم بياني غير حلقي.
- [x] **1.5 نظام الأوامر والتراجع (Undo/Redo & History Engine)**
  - [x] تعريف Trait للأوامر العكسية: `EditCommand`.
  - [x] تنفيذ الأوامر الأساسية: `AddNode`, `RemoveNode`, `Connect`, `Disconnect`, `MoveNodes`.
  - [x] تنفيذ المعاملات المركبة `CompoundTransaction` لحذف النودات مع أسلاكها والتراجع عنها كخطوة ذرية واحدة.
  - [x] بناء `HistoryStack` مع حدود الذاكرة وتفريغ الـ Redo عند التعديل الجديد.
- [x] **1.6 اختبارات الوحدات الشاملة (Unit Tests)**
  - [x] اختبار إنشاء النودات والمنافذ.
  - [x] اختبار مصفوفة التحقق وسعات المنافذ.
  - [x] اختبار منع الحلقات المغلقة (Cycle Detection).
  - [x] اختبار الـ Undo والـ Redo والمعاملات المركبة (نجاح بنسبة 100%).

---

## 🟢 المرحلة 2: نظام الأوامر الموحد ولوحة البحث (Unified Commands & Palette)
> **المرجع:** `06_unified_command_system.md`
> **الحالة:** مكتملة بنسبة 100% ومختبرة بنجاح ✅

- [x] **2.1 سجل الأوامر المركزي (Command Registry)**
  - [x] تعريف `CommandDescriptor` (المعرف، العنوان، التصنيف، الأيقونة، الاختصار).
  - [x] بناء `CommandRegistry` مع الفهرسة والبحث النصي (Fuzzy/Substring Search).
- [x] **2.2 موجه الأوامر المركزي (Command Dispatcher)**
  - [x] استقبال طلبات التنفيذ والتحقق من السياق عبر `CommandContext`.
  - [x] التوجيه التلقائي لمكدس `HistoryStack` وتكامل الـ Undo/Redo.
- [x] **2.3 محرك الاختصارات الديناميكي (Keybinding Manager)**
  - [x] فك ارتباط الكيبورد بالأكواد وربط `KeyChord` بـ `CommandId` عبر `EditorKey`.
  - [x] دعم حفظ وتحميل وتوليد الاختصارات من وإلى صيغة RON (`keybindings.ron`).
- [x] **2.4 لوحة الأوامر السريعة (Command Palette Core)**
  - [x] محرك البحث الذكي في سجل الأوامر تمهيداً للربط مع `bevy_luma_widgets::Modal`.
- [x] **2.5 سجل الحماية من الانهيار (Crash Journaling)**
  - [x] تسجيل الأوامر المتسلسلة زمنياً في `CrashJournal` بالذاكرة أو القرص (`session_journal.log`).
  - [x] اختبارات وحدات شاملة لنظام الأوامر (نجاح بنسبة 100%).

---

## 🟢 المرحلة 3: فضاء العمل والكانفاس التفاعلي (Canvas & Interactions)
> **المرجع:** `04_canvas_navigation_and_focus.md`، `08_ui_stack_and_luma_integration.md`
> **الحالة:** مكتملة بنسبة 100% ومختبرة بنجاح ✅

- [x] **3.1 إضافة المحرر لـ Bevy (LumaEditorPlugin)**
  - [x] تسجيل كافة موارد المحرر الثمانية (`GraphStore`, `HistoryStack`, `IdGenerator`, `CanvasTransform`, `SelectionState`, `WiringState`, `KeybindingMap`, `CommandDispatcher`) كـ Bevy Resources.
- [x] **3.2 تحويل الإحداثيات والملاحة (Pan & Zoom Math)**
  - [x] هيكل `CanvasTransform` للتحويل الدقيق بين Screen Space و Canvas Space.
  - [x] ميزة التكبير المتمركز حول المؤشر (Zoom to Cursor) مع المحافظة على النقطة المركزية.
  - [x] أنظمة التحكم بالـ Pan عبر زر الفأرة الأوسط أو مسطرة المسافة + السحب.
- [x] **3.3 حالة التوصيل والأسلاك (Wiring State & Splines)**
  - [x] آلة حالة التوصيل `WiringState` (`Idle`, `DraggingNew`, `Reconnecting`).
  - [x] خوارزميات حساب مماسات منحنيات بيزيير التكعيبية (Cubic Bézier Splines) وتوليد النقاط.
- [x] **3.4 نظام التحديد (Selection System)**
  - [x] هيكل `SelectionState` لإدارة التحديد المفرد والإضافي (`Shift`) وصناديق التحديد.
- [x] **3.5 ربط اختصارات لوحة المفاتيح بالمحرك**
  - [x] نظام `canvas_shortcuts_system` لمعالجة ضغطات المفاتيح وإرسالها للموجه المركزي.
- [x] **3.6 اختبارات وحدات شاملة للكانفاس (100% نجاح في `tests/canvas_tests.rs`).**

---

## 🟢 المرحلة 4: بطاقات النودات البصرية (Visual Node Cards)
> **المرجع:** `05_node_lifecycle_and_extensibility.md`، `08_ui_stack_and_luma_integration.md`
> **الحالة:** مكتملة بنسبة 100% ومختبرة بنجاح ✅

- [x] **4.1 بطاقة النود وهيكلها البصري (Card Layout)**
  - [x] هيكل `NodeCardLayout` لحساب أبعاد البطاقة وتوزيع المنافذ ومحاذاة المقابس على الحواف.
  - [x] تكامل مباشر مع `bevy_luma_widgets::card::Card`.
- [x] **4.2 تفاعل سحب وتحريك النودات (Node Dragging)**
  - [x] هيكل `NodeDragState` لإدارة التقاط وتحريك عدة نودات متزامنة في فضاء الكانفاس.
  - [x] دمج الحركة وتسجيل أمر `MoveNodesCommand` في التاريخ عند الإفلات فقط (Debounced) مع دعم التراجع الكامل.
- [x] **4.3 قائمة الإضافة السريعة (Node Spawn Palette)**
  - [x] هيكل `NodePalette` وقوالب النودات المسبقة (`AssetLoader`, `MeshNode`, `MaterialNode`, `EntityAssembler`).
  - [x] توليد النودات مع كافة منافذها ضمن معاملة ذرية واحدة (`CompoundTransaction`).
- [x] **4.4 اختبارات وحدات شاملة للنودات (100% نجاح في `tests/node_tests.rs`).**

---

## 🟢 المرحلة 5: نظام الغوص وعزل الأدوات (Contextual Focus / "Dive-In")
> **المرجع:** `00_architecture_overview.md`، `04_canvas_navigation_and_focus.md`
> **الحالة:** مكتملة بنسبة 100% ومختبرة بنجاح ✅

- [x] **5.1 إدارة مكدس التنقل الهرمي (Navigation State)**
  - [x] هيكل `WorkspaceContext` (`RootGraph` مقابل `FocusedNode { node_id, node_kind, node_title }`).
  - [x] إدارة مكدس التنقل `NavigationState` مع دعم الغوص المتداخل وحساب العمق.
- [x] **5.2 مسار التنقل العلوي (Breadcrumbs Navigation)**
  - [x] توليد مسارات الـ Breadcrumbs تلقائياً (`["Root Graph", "Hero Model", "Armor Shader"]`).
  - [x] دعم التراجع للخلف خطوة بخطوة أو التصفير للجذر `reset_to_root`.
- [x] **5.3 شريط الأدوات الموجه بالسياق (Contextual Tool Registry)**
  - [x] هيكل `ContextualTool` وسجل الأدوات `ContextualToolRegistry`.
  - [x] أطقم أدوات افتراضية مسبقة لـ `MeshNode`, `MaterialNode`, `PhysicsNode`, و `AssetLoader`.
- [x] **5.4 العزل الصارم للأدوات (Strict Tool Isolation)**
  - [x] ضمان عدم تسرب أدوات الشيدر أو الفيزياء داخل نود المجسمات (والعكس)، لتحقيق أقصى تركيز للمطور.
- [x] **5.5 اختبارات وحدات شاملة لنظام الغوص (100% نجاح في `tests/focus_tests.rs`).**

---

## 🟢 المرحلة 6: العقد التخصصية والمعاينات المصغرة (Domain Nodes & Viewports)
> **المرجع:** `05_node_lifecycle_and_extensibility.md`
> **الحالة:** مكتملة بنسبة 100% ومختبرة بنجاح ✅

- [x] **6.1 حزم بيانات النودات التخصصية (Domain Payloads)**
  - [x] هياكل `AssetLoaderData`, `MeshData`, `MaterialData`, و `EntityAssemblerData`.
  - [x] حاوية `DomainPayloadStore` لتخزين وتعديل المعطيات ثلاثية الأبعاد.
- [x] **6.2 معمارية وترشيد أداء المعاينات المصغرة (Viewport Optimization)**
  - [x] هيكل `NodeViewportTarget` ومدير المعاينات `ViewportManager`.
  - [x] دعم ترشيد استهلاك كارت الشاشة: تخفيض الإطارات للنودات البعيدة (FPS Throttling)، التحقق من اتساخ البيانات (Dirty Flag)، وإخفاء الكاميرات غير المرئية (Frustum Culling).
- [x] **6.3 محرك تجميع الكيانات ثلاثية الأبعاد (Graph Dependency Evaluation)**
  - [x] تتبع مسار الأسلاك العكسي من نود التجميع `EntityAssembler` إلى نودات `MeshNode`, `AssetLoader`, و `MaterialNode`.
  - [x] توليد كيان المشهد المكتمل `AssembledSceneEntity` (الاسم، الـ Transform، مسار المجسم، وخامة الـ PBR).
- [x] **6.4 اختبارات وحدات شاملة للأنظمة التخصصية (100% نجاح في `tests/domain_tests.rs`).**

---

## 🟢 المرحلة 7: حفظ وتصدير المشاهد (Serialization & BSN Integration)
> **المرجع:** `07_file_formats_and_bsn_integration.md`
> **الحالة:** مكتملة بنسبة 100% ومختبرة بنجاح ✅

- [x] **7.1 حفظ وتحميل ملفات الـ Graph (`.luma` RON Format)**
  - [x] هيكل `LumaGraphFile` و `LumaGraphMeta` لحفظ كامل المشهد والكانفاس والمعطيات التخصصية.
  - [x] دعم الحفظ والقراءة النصية بصيغة RON مع استعادة الفهارس والروابط بدقة.
- [x] **7.2 مترجم الغراف لـ BSN (The Graph Compiler / Baker)**
  - [x] هيكل `BsnCompiler` لفرز وترجمة كائنات الـ Graph إلى صيغة مشهد Bevy الأصلية (`.bsn`).
  - [x] توليد المكونات القياسية (`Name`, `Transform`, `Mesh3d`, `MeshMaterial3d`) بدون أي تكلفة وقت التشغيل (Zero Runtime Overhead).
- [x] **7.3 المزامنة التلقائية (Auto-Bake on Save)**
  - [x] خط أنابيب `AutoBakePipeline` الذي يحفظ ملف المصدر `.luma` ويولد في نفس اللحظة ملف `.bsn` للعبة.
- [x] **7.4 اختبارات وحدات شاملة لنظام الحفظ والتصدير (100% نجاح في `tests/scene_tests.rs`).**
