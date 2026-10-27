use std::collections::HashSet;

use luma_editor::core::graph::{GraphStore, Node, Pin, PinCapacity, PinDirection};
use luma_editor::core::id::IdGenerator;
use luma_editor::dataflow::engine::DataflowEngine;
use luma_editor::dataflow::registry::NodeRegistry;
use luma_editor::dataflow::types::{coerce, DataType};
use luma_editor::dataflow::value::{LinearColor, PrimitiveValue, Value};
use luma_editor::nodes::stdlib::register_stdlib;

#[test]
fn test_value_total_eq_nan_immunity() {
    let nan_a = PrimitiveValue::Float(f32::NAN);
    let nan_b = PrimitiveValue::Float(f32::NAN);
    assert!(nan_a.total_eq(&nan_b), "NaN must equal NaN in total_eq to prevent infinite dirty loops!");

    let zero_pos = PrimitiveValue::Float(0.0);
    let zero_neg = PrimitiveValue::Float(-0.0);
    assert!(zero_pos.total_eq(&zero_neg), "+0.0 must equal -0.0 in total_eq!");

    let vec_nan_a = PrimitiveValue::Vec3(glam::Vec3::new(1.0, f32::NAN, 3.0));
    let vec_nan_b = PrimitiveValue::Vec3(glam::Vec3::new(1.0, f32::NAN, 3.0));
    assert!(vec_nan_a.total_eq(&vec_nan_b), "Vectors containing NaN must equal each other!");

    let diff_a = PrimitiveValue::Float(1.5);
    let diff_b = PrimitiveValue::Float(2.5);
    assert!(!diff_a.total_eq(&diff_b));
}

#[test]
fn test_resource_id_aba_immunity() {
    let res_a = Value::new_resource(DataType::Mesh, 100, "MeshA".to_string());
    let id_a = match &res_a {
        Value::Resource { id, .. } => *id,
        _ => panic!("Expected Resource"),
    };

    drop(res_a); // Dropped

    let res_b = Value::new_resource(DataType::Mesh, 100, "MeshB".to_string());
    let id_b = match &res_b {
        Value::Resource { id, .. } => *id,
        _ => panic!("Expected Resource"),
    };

    assert_ne!(id_a, id_b, "Resource IDs must be strictly monotonic to prevent ABA cache hits!");
}

#[test]
fn test_coercion_rules() {
    // 1. Int -> Float
    let int_val = Value::int(42);
    let coerced_float = coerce(&int_val, &DataType::Float).expect("Should coerce Int to Float");
    assert_eq!(coerced_float.fingerprint(), Value::float(42.0).fingerprint());

    // 2. Float -> Vec3 Broadcast
    let float_val = Value::float(7.5);
    let coerced_vec = coerce(&float_val, &DataType::Vec3).expect("Should broadcast Float to Vec3");
    assert_eq!(coerced_vec.fingerprint(), Value::vec3(glam::Vec3::splat(7.5)).fingerprint());

    // 3. Color <-> Vec4
    let color_val = Value::color(LinearColor::new(0.1, 0.2, 0.3, 1.0));
    let coerced_vec4 = coerce(&color_val, &DataType::Vec4).expect("Should coerce Color to Vec4");
    assert_eq!(coerced_vec4.fingerprint(), Value::vec4(glam::Vec4::new(0.1, 0.2, 0.3, 1.0)).fingerprint());

    // 4. Scalar -> List Promotion
    let coerced_list = coerce(&float_val, &DataType::List(Box::new(DataType::Float)))
        .expect("Should promote scalar to List");
    assert!(coerced_list.is_heavy_resource());

    // 5. List -> Scalar MUST BE STRICTLY REJECTED!
    let rejected = coerce(&coerced_list, &DataType::Float);
    assert!(rejected.is_none(), "List -> Scalar must be strictly rejected to prevent silent data loss!");
}

#[test]
fn test_math_dataflow_execution() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    let mut engine = DataflowEngine::new(registry);

    // 1. Create math.add node: (A + B)
    let add_id = ids.next_node_id();
    let add_in0 = ids.next_pin_id();
    let add_in1 = ids.next_pin_id();
    let add_out = ids.next_pin_id();

    let mut add_node = Node::new(add_id, "Add Node", "math.add", glam::Vec2::ZERO);
    add_node.inputs = vec![add_in0, add_in1];
    add_node.outputs = vec![add_out];
    graph.add_node(add_node);

    graph.add_pin(Pin {
        id: add_in0,
        node_id: add_id,
        name: "a".into(),
        direction: PinDirection::Input,
        data_type: luma_editor::core::DataType::new("Float"),
        capacity: PinCapacity::Single,
    }).unwrap();
    graph.add_pin(Pin {
        id: add_in1,
        node_id: add_id,
        name: "b".into(),
        direction: PinDirection::Input,
        data_type: luma_editor::core::DataType::new("Float"),
        capacity: PinCapacity::Single,
    }).unwrap();
    graph.add_pin(Pin {
        id: add_out,
        node_id: add_id,
        name: "sum".into(),
        direction: PinDirection::Output,
        data_type: luma_editor::core::DataType::new("Float"),
        capacity: PinCapacity::Multiple,
    }).unwrap();

    // 2. Create math.multiply node: (sum * C)
    let mul_id = ids.next_node_id();
    let mul_in0 = ids.next_pin_id();
    let mul_in1 = ids.next_pin_id();
    let mul_out = ids.next_pin_id();

    let mut mul_node = Node::new(mul_id, "Multiply Node", "math.multiply", glam::Vec2::ZERO);
    mul_node.inputs = vec![mul_in0, mul_in1];
    mul_node.outputs = vec![mul_out];
    graph.add_node(mul_node);

    graph.add_pin(Pin {
        id: mul_in0,
        node_id: mul_id,
        name: "a".into(),
        direction: PinDirection::Input,
        data_type: luma_editor::core::DataType::new("Float"),
        capacity: PinCapacity::Single,
    }).unwrap();
    graph.add_pin(Pin {
        id: mul_in1,
        node_id: mul_id,
        name: "b".into(),
        direction: PinDirection::Input,
        data_type: luma_editor::core::DataType::new("Float"),
        capacity: PinCapacity::Single,
    }).unwrap();
    graph.add_pin(Pin {
        id: mul_out,
        node_id: mul_id,
        name: "product".into(),
        direction: PinDirection::Output,
        data_type: luma_editor::core::DataType::new("Float"),
        capacity: PinCapacity::Multiple,
    }).unwrap();

    // Wire: add_out -> mul_in0
    graph.connect(ids.next_connection_id(), add_out, mul_in0).unwrap();

    // Set parameters: Add inputs = (10.0 + 5.0 = 15.0), Mul input b = 4.0 -> Result = 60.0
    engine.set_pin_value(&graph, add_in0, PrimitiveValue::Float(10.0));
    engine.set_pin_value(&graph, add_in1, PrimitiveValue::Float(5.0));
    engine.set_pin_value(&graph, mul_in1, PrimitiveValue::Float(4.0));

    // Pull result
    let pinned = HashSet::new();
    let outputs = engine.pull_node(&graph, mul_id, &pinned).expect("Should compute graph");
    assert_eq!(outputs.len(), 1);

    match &outputs[0] {
        Value::Primitive(PrimitiveValue::Float(res)) => {
            assert_eq!(*res, 60.0, "(10 + 5) * 4 must equal 60.0");
        }
        _ => panic!("Expected Float result"),
    }
}

#[test]
fn test_lru_memory_budget_eviction() {
    let mut store = luma_editor::dataflow::cache::EngineCacheStore::new(1000); // 1 KB budget
    let pinned = HashSet::new();

    let node_1 = luma_editor::core::id::NodeId(1);
    let node_2 = luma_editor::core::id::NodeId(2);

    // Node 1 produces 600 bytes
    let val_1 = Value::new_resource(DataType::Mesh, 600, vec![1u8; 600]);
    store.update_outputs(node_1, vec![val_1], vec![101], &pinned);
    assert_eq!(store.current_resource_bytes, 600);

    store.advance_tick();

    // Node 2 produces 600 bytes -> total 1200 bytes > 1000 bytes budget!
    let val_2 = Value::new_resource(DataType::Mesh, 600, vec![2u8; 600]);
    store.update_outputs(node_2, vec![val_2], vec![102], &pinned);

    // Oldest node_1 heavy resource must be evicted!
    assert!(store.current_resource_bytes <= 1000);
    assert_eq!(store.caches.get(&node_1).unwrap().heavy_resource_bytes(), 0);
    assert_eq!(store.caches.get(&node_2).unwrap().heavy_resource_bytes(), 600);
}

#[test]
fn test_complex_math_pipeline_dataflow_assembly() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);
    let mut engine = DataflowEngine::new(registry);

    // 1. math.vector_compose (x=3.0, y=4.0, z=0.0) -> Vec3(3, 4, 0)
    let comp_id = ids.next_node_id();
    let comp_in_x = ids.next_pin_id();
    let comp_in_y = ids.next_pin_id();
    let comp_in_z = ids.next_pin_id();
    let comp_out = ids.next_pin_id();
    let mut comp_node = Node::new(comp_id, "Compose", "math.vector_compose", glam::Vec2::ZERO);
    comp_node.inputs = vec![comp_in_x, comp_in_y, comp_in_z];
    comp_node.outputs = vec![comp_out];
    graph.add_node(comp_node);
    graph.add_pin(Pin { id: comp_in_x, node_id: comp_id, name: "x".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: comp_in_y, node_id: comp_id, name: "y".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: comp_in_z, node_id: comp_id, name: "z".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: comp_out, node_id: comp_id, name: "vector".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Vec3"), capacity: PinCapacity::Multiple }).unwrap();

    // 2. math.vector_length (vector) -> length = 5.0
    let len_id = ids.next_node_id();
    let len_in = ids.next_pin_id();
    let len_out = ids.next_pin_id();
    let mut len_node = Node::new(len_id, "Length", "math.vector_length", glam::Vec2::ZERO);
    len_node.inputs = vec![len_in];
    len_node.outputs = vec![len_out];
    graph.add_node(len_node);
    graph.add_pin(Pin { id: len_in, node_id: len_id, name: "vector".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Vec3"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: len_out, node_id: len_id, name: "length".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    // 3. math.multiply (a=length, b=2.0) -> product = 10.0
    let mul_id = ids.next_node_id();
    let mul_in_a = ids.next_pin_id();
    let mul_in_b = ids.next_pin_id();
    let mul_out = ids.next_pin_id();
    let mut mul_node = Node::new(mul_id, "Multiply", "math.multiply", glam::Vec2::ZERO);
    mul_node.inputs = vec![mul_in_a, mul_in_b];
    mul_node.outputs = vec![mul_out];
    graph.add_node(mul_node);
    graph.add_pin(Pin { id: mul_in_a, node_id: mul_id, name: "a".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: mul_in_b, node_id: mul_id, name: "b".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: mul_out, node_id: mul_id, name: "result".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    // 4. math.power (base=product, exponent=2.0) -> result = 100.0
    let pow_id = ids.next_node_id();
    let pow_in_base = ids.next_pin_id();
    let pow_in_exp = ids.next_pin_id();
    let pow_out = ids.next_pin_id();
    let mut pow_node = Node::new(pow_id, "Power", "math.power", glam::Vec2::ZERO);
    pow_node.inputs = vec![pow_in_base, pow_in_exp];
    pow_node.outputs = vec![pow_out];
    graph.add_node(pow_node);
    graph.add_pin(Pin { id: pow_in_base, node_id: pow_id, name: "base".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: pow_in_exp, node_id: pow_id, name: "exponent".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: pow_out, node_id: pow_id, name: "result".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    // Connect wires:
    // comp.vector -> len.vector
    graph.connect(ids.next_connection_id(), comp_out, len_in).unwrap();
    // len.length -> mul.a
    graph.connect(ids.next_connection_id(), len_out, mul_in_a).unwrap();
    // mul.result -> pow.base
    graph.connect(ids.next_connection_id(), mul_out, pow_in_base).unwrap();

    // Set parameters: (3, 4, 0) vector -> len=5 -> * 2 = 10 -> ^ 2 = 100
    engine.set_pin_value(&graph, comp_in_x, PrimitiveValue::Float(3.0));
    engine.set_pin_value(&graph, comp_in_y, PrimitiveValue::Float(4.0));
    engine.set_pin_value(&graph, comp_in_z, PrimitiveValue::Float(0.0));
    engine.set_pin_value(&graph, mul_in_b, PrimitiveValue::Float(2.0));
    engine.set_pin_value(&graph, pow_in_exp, PrimitiveValue::Float(2.0));

    // Pull terminal node: pow_id
    let pinned = HashSet::new();
    let outputs = engine.pull_node(&graph, pow_id, &pinned).expect("Should evaluate multi-stage math pipeline");
    assert_eq!(outputs.len(), 1);

    match &outputs[0] {
        Value::Primitive(PrimitiveValue::Float(v)) => {
            assert_eq!(*v, 100.0, "((|(3, 4, 0)| * 2)^2 = (5 * 2)^2 = 100.0");
        }
        _ => panic!("Expected Float result"),
    }
}


fn eval_node(registry: &NodeRegistry, node_type: &str, inputs: &[Value]) -> Vec<Value> {
    let desc = registry.get(node_type).unwrap_or_else(|| panic!("Node type '{}' not found in registry", node_type));
    let mut outputs = vec![Value::none(); desc.outputs.len()];
    desc.execute_sync(inputs, &mut outputs).expect("Sync execution should succeed");
    outputs
}

fn get_float_res(outputs: &[Value], idx: usize) -> f32 {
    match &outputs[idx] {
        Value::Primitive(PrimitiveValue::Float(f)) => *f,
        other => panic!("Expected Float output at idx {}, got {:?}", idx, other),
    }
}

fn get_vec3_res(outputs: &[Value], idx: usize) -> glam::Vec3 {
    match &outputs[idx] {
        Value::Primitive(PrimitiveValue::Vec3(v)) => *v,
        other => panic!("Expected Vec3 output at idx {}, got {:?}", idx, other),
    }
}

#[test]
fn test_math_arithmetic_nodes() {
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    // add
    let out = eval_node(&registry, "math.add", &[Value::float(3.0), Value::float(4.0)]);
    assert_eq!(get_float_res(&out, 0), 7.0);

    // subtract
    let out = eval_node(&registry, "math.subtract", &[Value::float(10.0), Value::float(4.0)]);
    assert_eq!(get_float_res(&out, 0), 6.0);

    // multiply
    let out = eval_node(&registry, "math.multiply", &[Value::float(3.0), Value::float(5.0)]);
    assert_eq!(get_float_res(&out, 0), 15.0);

    // divide & safe divide by zero
    let out = eval_node(&registry, "math.divide", &[Value::float(10.0), Value::float(2.0)]);
    assert_eq!(get_float_res(&out, 0), 5.0);
    let out_zero = eval_node(&registry, "math.divide", &[Value::float(10.0), Value::float(0.0)]);
    assert_eq!(get_float_res(&out_zero, 0), 0.0);

    // modulo & safe modulo by zero
    let out = eval_node(&registry, "math.modulo", &[Value::float(10.0), Value::float(3.0)]);
    assert_eq!(get_float_res(&out, 0), 1.0);
    let out_zero = eval_node(&registry, "math.modulo", &[Value::float(10.0), Value::float(0.0)]);
    assert_eq!(get_float_res(&out_zero, 0), 0.0);

    // power
    let out = eval_node(&registry, "math.power", &[Value::float(2.0), Value::float(3.0)]);
    assert_eq!(get_float_res(&out, 0), 8.0);

    // sqrt
    let out = eval_node(&registry, "math.sqrt", &[Value::float(16.0)]);
    assert_eq!(get_float_res(&out, 0), 4.0);
    let out_neg = eval_node(&registry, "math.sqrt", &[Value::float(-9.0)]);
    assert_eq!(get_float_res(&out_neg, 0), 0.0);

    // abs
    let out = eval_node(&registry, "math.abs", &[Value::float(-7.5)]);
    assert_eq!(get_float_res(&out, 0), 7.5);

    // negate
    let out = eval_node(&registry, "math.negate", &[Value::float(5.0)]);
    assert_eq!(get_float_res(&out, 0), -5.0);

    // sign
    let out_pos = eval_node(&registry, "math.sign", &[Value::float(4.2)]);
    assert_eq!(get_float_res(&out_pos, 0), 1.0);
    let out_neg = eval_node(&registry, "math.sign", &[Value::float(-3.1)]);
    assert_eq!(get_float_res(&out_neg, 0), -1.0);
    let out_zero = eval_node(&registry, "math.sign", &[Value::float(0.0)]);
    assert_eq!(get_float_res(&out_zero, 0), 0.0);
}

#[test]
fn test_math_rounding_and_interpolation() {
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    // floor, ceil, round, fract
    assert_eq!(get_float_res(&eval_node(&registry, "math.floor", &[Value::float(3.7)]), 0), 3.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.ceil", &[Value::float(3.2)]), 0), 4.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.round", &[Value::float(3.5)]), 0), 4.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.round", &[Value::float(3.2)]), 0), 3.0);
    let fract_val = get_float_res(&eval_node(&registry, "math.fract", &[Value::float(3.75)]), 0);
    assert!((fract_val - 0.75).abs() < 1e-5);

    // min, max, clamp
    assert_eq!(get_float_res(&eval_node(&registry, "math.min", &[Value::float(4.0), Value::float(9.0)]), 0), 4.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.max", &[Value::float(4.0), Value::float(9.0)]), 0), 9.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.clamp", &[Value::float(15.0), Value::float(0.0), Value::float(10.0)]), 0), 10.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.clamp", &[Value::float(-5.0), Value::float(0.0), Value::float(10.0)]), 0), 0.0);

    // lerp
    let lerp_res = get_float_res(&eval_node(&registry, "math.lerp", &[Value::float(10.0), Value::float(20.0), Value::float(0.25)]), 0);
    assert_eq!(lerp_res, 12.5);

    // smoothstep
    let ss_res = get_float_res(&eval_node(&registry, "math.smoothstep", &[Value::float(0.0), Value::float(1.0), Value::float(0.5)]), 0);
    assert_eq!(ss_res, 0.5);

    // map_range
    let map_res = get_float_res(&eval_node(&registry, "math.map_range", &[
        Value::float(5.0),
        Value::float(0.0),
        Value::float(10.0),
        Value::float(100.0),
        Value::float(200.0),
    ]), 0);
    assert_eq!(map_res, 150.0);
}

#[test]
fn test_math_trig_and_exponents() {
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    // radians & degrees
    let rad = get_float_res(&eval_node(&registry, "math.radians", &[Value::float(180.0)]), 0);
    assert!((rad - std::f32::consts::PI).abs() < 1e-4);
    let deg = get_float_res(&eval_node(&registry, "math.degrees", &[Value::float(std::f32::consts::PI)]), 0);
    assert!((deg - 180.0).abs() < 1e-4);

    // sin & cos
    let sin_val = get_float_res(&eval_node(&registry, "math.sin", &[Value::float(std::f32::consts::FRAC_PI_2)]), 0);
    assert!((sin_val - 1.0).abs() < 1e-4);
    let cos_val = get_float_res(&eval_node(&registry, "math.cos", &[Value::float(0.0)]), 0);
    assert!((cos_val - 1.0).abs() < 1e-4);

    // atan2
    let atan_val = get_float_res(&eval_node(&registry, "math.atan2", &[Value::float(1.0), Value::float(1.0)]), 0);
    assert!((atan_val - std::f32::consts::FRAC_PI_4).abs() < 1e-4);

    // exp & log
    let exp_val = get_float_res(&eval_node(&registry, "math.exp", &[Value::float(1.0)]), 0);
    assert!((exp_val - std::f32::consts::E).abs() < 1e-4);
    let log_val = get_float_res(&eval_node(&registry, "math.log", &[Value::float(std::f32::consts::E)]), 0);
    assert!((log_val - 1.0).abs() < 1e-4);

    // log2 & log10
    let log2_val = get_float_res(&eval_node(&registry, "math.log2", &[Value::float(8.0)]), 0);
    assert!((log2_val - 3.0).abs() < 1e-4);
    let log10_val = get_float_res(&eval_node(&registry, "math.log10", &[Value::float(100.0)]), 0);
    assert!((log10_val - 2.0).abs() < 1e-4);
}

#[test]
fn test_math_comparison_nodes() {
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    assert_eq!(get_float_res(&eval_node(&registry, "math.greater_than", &[Value::float(5.0), Value::float(2.0)]), 0), 1.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.greater_than", &[Value::float(2.0), Value::float(5.0)]), 0), 0.0);

    assert_eq!(get_float_res(&eval_node(&registry, "math.less_than", &[Value::float(2.0), Value::float(5.0)]), 0), 1.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.less_than", &[Value::float(5.0), Value::float(2.0)]), 0), 0.0);

    assert_eq!(get_float_res(&eval_node(&registry, "math.equal", &[Value::float(4.0), Value::float(4.0)]), 0), 1.0);
    assert_eq!(get_float_res(&eval_node(&registry, "math.equal", &[Value::float(4.0), Value::float(4.5)]), 0), 0.0);
}

#[test]
fn test_math_vector_nodes() {
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    let v1 = glam::Vec3::new(1.0, 2.0, 3.0);
    let v2 = glam::Vec3::new(4.0, 5.0, 6.0);

    // vector_add
    let add_res = get_vec3_res(&eval_node(&registry, "math.vector_add", &[Value::vec3(v1), Value::vec3(v2)]), 0);
    assert_eq!(add_res, glam::Vec3::new(5.0, 7.0, 9.0));

    // vector_subtract
    let sub_res = get_vec3_res(&eval_node(&registry, "math.vector_subtract", &[Value::vec3(v2), Value::vec3(v1)]), 0);
    assert_eq!(sub_res, glam::Vec3::new(3.0, 3.0, 3.0));

    // vector_multiply (component-wise)
    let mul_res = get_vec3_res(&eval_node(&registry, "math.vector_multiply", &[
        Value::vec3(glam::Vec3::new(2.0, 3.0, 4.0)),
        Value::vec3(glam::Vec3::new(3.0, 2.0, 0.5)),
    ]), 0);
    assert_eq!(mul_res, glam::Vec3::new(6.0, 6.0, 2.0));

    // vector_scale
    let scale_res = get_vec3_res(&eval_node(&registry, "math.vector_scale", &[Value::vec3(v1), Value::float(2.0)]), 0);
    assert_eq!(scale_res, glam::Vec3::new(2.0, 4.0, 6.0));

    // vector_dot
    let dot_res = get_float_res(&eval_node(&registry, "math.vector_dot", &[
        Value::vec3(glam::Vec3::X),
        Value::vec3(glam::Vec3::Y),
    ]), 0);
    assert_eq!(dot_res, 0.0);

    // vector_cross
    let cross_res = get_vec3_res(&eval_node(&registry, "math.vector_cross", &[
        Value::vec3(glam::Vec3::X),
        Value::vec3(glam::Vec3::Y),
    ]), 0);
    assert_eq!(cross_res, glam::Vec3::Z);

    // vector_length
    let len_res = get_float_res(&eval_node(&registry, "math.vector_length", &[
        Value::vec3(glam::Vec3::new(3.0, 4.0, 0.0)),
    ]), 0);
    assert_eq!(len_res, 5.0);

    // vector_distance
    let dist_res = get_float_res(&eval_node(&registry, "math.vector_distance", &[
        Value::vec3(glam::Vec3::ZERO),
        Value::vec3(glam::Vec3::new(3.0, 4.0, 0.0)),
    ]), 0);
    assert_eq!(dist_res, 5.0);

    // vector_normalize
    let norm_res = get_vec3_res(&eval_node(&registry, "math.vector_normalize", &[
        Value::vec3(glam::Vec3::new(0.0, 10.0, 0.0)),
    ]), 0);
    assert_eq!(norm_res, glam::Vec3::Y);

    // compose: x, y, z -> Vec3
    let comp_res = get_vec3_res(&eval_node(&registry, "math.vector_compose", &[
        Value::float(7.0),
        Value::float(8.0),
        Value::float(9.0),
    ]), 0);
    assert_eq!(comp_res, glam::Vec3::new(7.0, 8.0, 9.0));

    // decompose: Vec3 -> x, y, z
    let decomp_res = eval_node(&registry, "math.vector_decompose", &[
        Value::vec3(glam::Vec3::new(7.0, 8.0, 9.0)),
    ]);
    assert_eq!(decomp_res.len(), 3);
    assert_eq!(get_float_res(&decomp_res, 0), 7.0);
    assert_eq!(get_float_res(&decomp_res, 1), 8.0);
    assert_eq!(get_float_res(&decomp_res, 2), 9.0);
}

#[test]
fn test_node_category_and_pin_shape_metadata() {
    use luma_editor::dataflow::descriptor::{NodeBodyStyle, NodeCategory, NodeDescriptor, PinShape};

    let desc = NodeDescriptor::new_sync("custom.vector_filter", "Vector Filter", "Physics/Forces", |_in, _out| Ok(()))
        .with_category(NodeCategory::new("Physics/Forces").with_tag("FORCE"))
        .with_accent_color(LinearColor::new(0.9, 0.4, 0.1, 1.0))
        .with_body_style(NodeBodyStyle::Empty)
        .with_input_full("impulse", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO), PinShape::Diamond)
        .with_output_full("force", DataType::Vec3, PinShape::Circle);

    assert_eq!(desc.ui.category.path, "Physics/Forces");
    assert_eq!(desc.ui.category.tag, "FORCE");
    assert_eq!(desc.ui.body_style, NodeBodyStyle::Empty);
    assert_eq!(desc.inputs[0].shape, PinShape::Diamond);
    assert_eq!(desc.outputs[0].shape, PinShape::Circle);
    assert!(desc.ui.accent_color.is_some());
}

#[test]
fn test_dynamic_palette_generation_from_registry() {
    use luma_editor::nodes::palette::NodePalette;

    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    let palette = NodePalette::from_registry(&registry);
    assert_eq!(palette.all().count(), registry.len() + 1); // +1 for Subgraph

    // Verify template derived from registry
    let add_tmpl = palette.get("math.add").expect("math.add must exist in palette");
    assert_eq!(add_tmpl.category.path, "Math/Arithmetic");
    assert_eq!(add_tmpl.category.tag, "MATH");
    assert_eq!(add_tmpl.pins.len(), 3); // 2 inputs + 1 output

    let vec_tmpl = palette.get("math.vector_add").expect("math.vector_add must exist in palette");
    assert_eq!(vec_tmpl.category.path, "Math/Vector");
    assert_eq!(vec_tmpl.category.tag, "VEC3");
}

#[test]
fn test_number_and_constant_nodes() {
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    // Test math.number source node
    let num_res = eval_node(&registry, "math.number", &[Value::float(42.5)]);
    assert_eq!(num_res.len(), 1);
    assert_eq!(get_float_res(&num_res, 0), 42.5);

    // Test math.integer source node
    let int_res = eval_node(&registry, "math.integer", &[Value::int(123)]);
    assert_eq!(int_res.len(), 1);
    match &int_res[0] {
        Value::Primitive(PrimitiveValue::Int(v)) => assert_eq!(*v, 123),
        _ => panic!("Expected Int result"),
    }

    // Test math.constant_pi
    let pi_res = eval_node(&registry, "math.constant_pi", &[]);
    assert_eq!(pi_res.len(), 1);
    assert_eq!(get_float_res(&pi_res, 0), std::f32::consts::PI);
}

#[test]
fn test_univis_style_card_layout() {
    use luma_editor::dataflow::descriptor::NodeBodyStyle;
    use luma_editor::nodes::card::NodeCardLayout;

    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();

    // 1. Parameter Node: math.number
    let num_id = ids.next_node_id();
    let num_in = ids.next_pin_id();
    let num_out = ids.next_pin_id();
    let mut num_node = Node::new(num_id, "Number", "math.number", glam::Vec2::ZERO);
    num_node.inputs = vec![num_in];
    num_node.outputs = vec![num_out];
    graph.add_node(num_node);
    graph.add_pin(Pin { id: num_in, node_id: num_id, name: "value".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: num_out, node_id: num_id, name: "output".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    let num_layout = NodeCardLayout::from_graph(num_id, &graph).expect("Layout must generate for math.number");
    assert_eq!(num_layout.category_tag, "NUM");
    assert_eq!(num_layout.body_style, NodeBodyStyle::Parameters);
    assert!(num_layout.number_value.is_some());
    assert_eq!(num_layout.size.x, 210.0);

    // 2. Pure Math Node: math.add
    let add_id = ids.next_node_id();
    let add_in_a = ids.next_pin_id();
    let add_in_b = ids.next_pin_id();
    let add_out = ids.next_pin_id();
    let mut add_node = Node::new(add_id, "Add", "math.add", glam::Vec2::ZERO);
    add_node.inputs = vec![add_in_a, add_in_b];
    add_node.outputs = vec![add_out];
    graph.add_node(add_node);
    graph.add_pin(Pin { id: add_in_a, node_id: add_id, name: "a".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: add_in_b, node_id: add_id, name: "b".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: add_out, node_id: add_id, name: "result".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    let add_layout = NodeCardLayout::from_graph(add_id, &graph).expect("Layout must generate for math.add");
    assert_eq!(add_layout.category_tag, "MATH");
    assert_eq!(add_layout.body_style, NodeBodyStyle::Empty);
    assert!(add_layout.number_value.is_none());
    assert_eq!(add_layout.size.x, 200.0);

    // Verify pin local positions
    let pos_in_a = add_layout.pin_local_position(add_in_a).unwrap();
    assert_eq!(pos_in_a.x, 0.0); // inputs on left edge
    let pos_out = add_layout.pin_local_position(add_out).unwrap();
    assert_eq!(pos_out.x, add_layout.size.x); // outputs on right edge
}

#[test]
fn test_live_io_values_and_custom_body_slot() {
    let ids = IdGenerator::new();
    let mut graph = GraphStore::new();
    let mut registry = NodeRegistry::new();
    register_stdlib(&mut registry);

    // Register a custom node with NodeBodyStyle::CustomSlot
    registry.register(
        luma_editor::dataflow::descriptor::NodeDescriptor::new_sync(
            "custom.display",
            "Custom Display",
            "Custom",
            |inputs, outputs| {
                outputs[0] = inputs[0].clone();
                Ok(())
            },
        )
        .with_body_style(luma_editor::dataflow::descriptor::NodeBodyStyle::CustomSlot)
        .with_input("in_val", DataType::Float, PrimitiveValue::Float(42.0))
        .with_output("out_val", DataType::Float),
    );

    let mut engine = DataflowEngine::new(registry);

    // 1. Create a Number node (outputs 10.0)
    let num_id = ids.next_node_id();
    let num_in = ids.next_pin_id();
    let num_out = ids.next_pin_id();
    let mut num_node = Node::new(num_id, "Number", "math.number", glam::Vec2::ZERO);
    num_node.inputs = vec![num_in];
    num_node.outputs = vec![num_out];
    graph.add_node(num_node);
    graph.add_pin(Pin { id: num_in, node_id: num_id, name: "value".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: num_out, node_id: num_id, name: "output".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    // 2. Create Add node (a + b)
    let add_id = ids.next_node_id();
    let add_in_a = ids.next_pin_id();
    let add_in_b = ids.next_pin_id();
    let add_out = ids.next_pin_id();
    let mut add_node = Node::new(add_id, "Add", "math.add", glam::Vec2::ZERO);
    add_node.inputs = vec![add_in_a, add_in_b];
    add_node.outputs = vec![add_out];
    graph.add_node(add_node);
    graph.add_pin(Pin { id: add_in_a, node_id: add_id, name: "a".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: add_in_b, node_id: add_id, name: "b".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: add_out, node_id: add_id, name: "result".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    // Connect num.output -> add.a
    let conn_id = ids.next_connection_id();
    graph.connect(conn_id, num_out, add_in_a).unwrap();

    // Set pin parameter values
    engine.set_pin_value(&graph, num_in, PrimitiveValue::Float(12.5));
    engine.set_pin_value(&graph, add_in_b, PrimitiveValue::Float(7.5));

    // Pull Add node to execute computation
    let pinned = HashSet::new();
    let outputs = engine.pull_node(&graph, add_id, &pinned).expect("Evaluation must succeed");
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].to_concise_string(), "20.0");

    // Test resolve_input_value
    // Wired pin: add.inputs[0] should resolve to num's output (12.5)
    let resolved_in_a = engine.resolve_input_value(&graph, add_id, 0);
    assert_eq!(resolved_in_a.to_concise_string(), "12.5");

    // Unwired pin with pin_values override: add.inputs[1] should resolve to 7.5
    let resolved_in_b = engine.resolve_input_value(&graph, add_id, 1);
    assert_eq!(resolved_in_b.to_concise_string(), "7.5");

    // Test get_cached_outputs
    let cached = engine.get_cached_outputs(add_id).expect("Outputs must be cached");
    assert_eq!(cached[0].to_concise_string(), "20.0");

    // Test CustomSlot layout
    let cust_id = ids.next_node_id();
    let cust_in = ids.next_pin_id();
    let cust_out = ids.next_pin_id();
    let mut cust_node = Node::new(cust_id, "Display", "custom.display", glam::Vec2::ZERO);
    cust_node.inputs = vec![cust_in];
    cust_node.outputs = vec![cust_out];
    graph.add_node(cust_node);
    graph.add_pin(Pin { id: cust_in, node_id: cust_id, name: "in_val".into(), direction: PinDirection::Input, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Single }).unwrap();
    graph.add_pin(Pin { id: cust_out, node_id: cust_id, name: "out_val".into(), direction: PinDirection::Output, data_type: luma_editor::core::DataType::new("Float"), capacity: PinCapacity::Multiple }).unwrap();

    let cust_layout = luma_editor::nodes::card::NodeCardLayout::from_graph(cust_id, &graph).expect("Layout must generate");
    assert_eq!(cust_layout.body_style, luma_editor::dataflow::descriptor::NodeBodyStyle::CustomSlot);
    assert_eq!(cust_layout.size.x, 230.0);
}

#[test]
fn test_top_menu_bar_structure_and_state() {
    use luma_editor::ui::menu_bar::{EditorMenuState, MenuKind, MenuItemAction};

    let mut menu_state = EditorMenuState::default();
    assert_eq!(menu_state.open_menu, None);

    // Open File menu
    menu_state.open_menu = Some(MenuKind::File);
    assert_eq!(menu_state.open_menu.unwrap().label(), "File");

    // Switch to Edit menu
    menu_state.open_menu = Some(MenuKind::Edit);
    assert_eq!(menu_state.open_menu.unwrap().label(), "Edit");

    // Switch to View menu
    menu_state.open_menu = Some(MenuKind::View);
    assert_eq!(menu_state.open_menu.unwrap().label(), "View");

    // Switch to Help menu
    menu_state.open_menu = Some(MenuKind::Help);
    assert_eq!(menu_state.open_menu.unwrap().label(), "Help");

    // Close menu
    menu_state.open_menu = None;
    assert_eq!(menu_state.open_menu, None);

    // Verify MenuItemAction variants are distinct
    assert_ne!(MenuItemAction::FileNew, MenuItemAction::FileSaveAndBake);
    assert_ne!(MenuItemAction::EditUndo, MenuItemAction::EditRedo);
    assert_ne!(MenuItemAction::ViewTogglePalette, MenuItemAction::ViewResetZoom);
}






