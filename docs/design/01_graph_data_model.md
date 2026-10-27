# 01 - نموذج بيانات الـ Graph (Graph Data Model)

تعتبر هذه الوثيقة المرجع الأساسي لهياكل البيانات الصافية (Pure Data Structures) المسؤولة عن تمثيل الـ Graph في الذاكرة دون أي ارتباط بـ UI.

---

## 1. المعرفات الفريدة القوية (Strong-Typed IDs)

لتجنب الخلط بين المعرفات (Primitive Obsession) وضمان الأمان أثناء التعديل، يتم استخدام معرّفات مخصصة كـ Tuples جديدة:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PinId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConnectionId(pub u64);
```

> **ملاحظة:** يمكن توليد المعرفات باستخدام عداد أحادي التزايد (Atomic Counter) أو مولد معرّفات عشوائية (UUID v4 / NanoId).

---

## 2. نظام المنافذ (Pins & Sockets)

كل عقدة تتواصل مع غيرها عبر منافذ (Pins). يمتلك كل منفذ اتجاهاً ونوعاً وقواعد سعة.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataType {
    pub name: String,
    // معرّف فريد لنوع البيانات لضمان توافق الأسلاك
    pub type_id: std::any::TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinCapacity {
    Single,   // يقبل توصيلاً واحداً فقط (شائع في الـ Inputs)
    Multiple, // يقبل توصيلات غير محدودة (شائع في الـ Outputs)
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub id: PinId,
    pub node_id: NodeId,
    pub name: String,
    pub direction: PinDirection,
    pub data_type: DataType,
    pub capacity: PinCapacity,
}
```

---

## 3. التوصيلات (Connections / Wires)

يمثل السلك علاقة مباشرة موجهة بين منفذ مخرج (Output) ومنفذ مدخل (Input):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub id: ConnectionId,
    pub source_node: NodeId,
    pub source_pin: PinId,
    pub target_node: NodeId,
    pub target_pin: PinId,
}
```

---

## 4. بنية العقدة المجردة (Node Base)

تحتوي العقدة في هذه الطبقة على معلومات الموقع والحالة فقط، بينما المنطق المخصص ينفصل في طبقة التمديد (Extensibility Layer):

```rust
use glam::Vec2; // أو Bevy Vec2

#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub kind: String, // نوع النود (مثلاً: "MeshLoader", "MaterialPBR")
    
    // فضاء إحداثيات الـ Canvas
    pub position: Vec2,
    pub size: Vec2,
    
    // قائمة المعرفات للمنافذ التابعة لها
    pub inputs: Vec<PinId>,
    pub outputs: Vec<PinId>,
    
    // إمكانية طي النود أو إخفاء التفاصيل
    pub is_collapsed: bool,
}
```

---

## 5. حاوية الـ Graph الرئيسية (`GraphStore`)

`GraphStore` هو الكيان الذي يملك كل العناصر ويوفر واجهة برمجية آمنة للقراءة والتعديل:

```rust
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct GraphStore {
    nodes: HashMap<NodeId, Node>,
    pins: HashMap<PinId, Pin>,
    connections: HashMap<ConnectionId, Connection>,
    
    // فهارس تسريع البحث (Fast Lookups)
    pin_connections: HashMap<PinId, Vec<ConnectionId>>,
    node_connections: HashMap<NodeId, Vec<ConnectionId>>,
}
```

### الدوال الأساسية للحاوية:
* `add_node(node: Node) -> NodeId`
* `remove_node(node_id: NodeId) -> Option<(Node, Vec<Connection>)>` (تُرجع النود وكل الأسلاك التي كانت متصلة به).
* `add_pin(pin: Pin) -> PinId`
* `remove_pin(pin_id: PinId) -> Option<Pin>`
* `connect(source_pin: PinId, target_pin: PinId) -> Result<ConnectionId, ConnectionError>`
* `disconnect(connection_id: ConnectionId) -> Option<Connection>`
* `get_incoming_connections(pin_id: PinId) -> &[ConnectionId]`
* `get_outgoing_connections(pin_id: PinId) -> &[ConnectionId]`
