#[allow(non_snake_case)]
#[allow(unsafe_code)]
#[allow(unused)]

#[allow(unused)]
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_double, c_void};
use std::ptr;
use std::slice;
use std::sync::Mutex;

use crate::wormgraph_core::{WormGraph, WormNode, NodeMetadata, FoundingFather, WormGraphError, Hash256};

// =============================================================================
// C API - FFI para C/C++/Python via ctypes
// =============================================================================

/// Opaque pointer para WormGraph
pub struct WormGraphHandle {
    graph: Mutex<WormGraph>,
}

/// Cria um novo WormGraph
/// @param max_tokens: capacidade maxima de tokens do contexto (2^21 = 2097152)
/// @param embedding_dim: dimensao dos embeddings (768, 1024, ou 4096)
/// @return handle opaco ou null em erro
#[no_mangle]
pub extern "C" fn wormgraph_create(max_tokens: usize, embedding_dim: usize) -> *mut WormGraphHandle {
    let graph = WormGraph::new(max_tokens).with_embedding_dim(embedding_dim);
    let handle = Box::new(WormGraphHandle {
        graph: Mutex::new(graph),
    });
    Box::into_raw(handle)
}

/// Destroi um WormGraph
#[no_mangle]
pub extern "C" fn wormgraph_destroy(handle: *mut WormGraphHandle) {
    if !handle.is_null() {
        unsafe { let _ = Box::from_raw(handle); }
    }
}

/// Adiciona um nó ao WormGraph
/// @param handle: handle do WormGraph
/// @param content: conteudo textual (null-terminated)
/// @param substrate_id: ID do substrato (null-terminated)
/// @param tags: tags separadas por virgula (null-terminated)
/// @param dna_flags: bitmask dos FoundingFathers (bits 0-11)
/// @param embedding: array de floats (embedding_dim elementos)
/// @param embedding_dim: dimensao do embedding
/// @param out_node_id: buffer de saida de 32 bytes para o NodeId
/// @return 0 em sucesso, codigo de erro negativo em falha
#[no_mangle]
pub extern "C" fn wormgraph_add_node(
    handle: *mut WormGraphHandle,
    content: *const c_char,
    substrate_id: *const c_char,
    tags: *const c_char,
    dna_flags: c_int,
    embedding: *const c_double,
    embedding_dim: usize,
    out_node_id: *mut u8,
) -> c_int {
    if handle.is_null() || content.is_null() || substrate_id.is_null() || embedding.is_null() || out_node_id.is_null() {
        return -1;
    }

    let handle = unsafe { &*handle };
    let mut graph = match handle.graph.lock() {
        Ok(g) => g,
        Err(_) => return -2,
    };

    let content = unsafe { CStr::from_ptr(content).to_string_lossy().into_owned() };
    let substrate_id = unsafe { CStr::from_ptr(substrate_id).to_string_lossy().into_owned() };
    let tags_str = unsafe { CStr::from_ptr(tags).to_string_lossy().into_owned() };
    let tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();

    let embedding_slice = unsafe { slice::from_raw_parts(embedding, embedding_dim) };
    let embedding: Vec<f32> = embedding_slice.iter().map(|&x| x as f32).collect();

    let dna = FoundingFather::ALL.iter()
        .enumerate()
        .filter(|(i, _)| (dna_flags >> i) & 1 == 1)
        .map(|(_, ff)| *ff)
        .collect();

    let metadata = NodeMetadata {
        substrate_id,
        phi_c: 0.99,
        theosis: 0.98,
        tags,
        cross_links: vec![],
        version: String::from("5.2.0"),
        seal: [0u8; 32],
        timestamp_ns: 1_000_000_000,
    };

    match graph.add_node(&content, metadata, dna, embedding) {
        Ok(node_id) => {
            unsafe {
                ptr::copy_nonoverlapping(node_id.as_ptr(), out_node_id, 32);
            }
            0
        }
        Err(_) => -3,
    }
}

/// Busca um nó por ID
#[no_mangle]
pub extern "C" fn wormgraph_get_node(
    handle: *mut WormGraphHandle,
    node_id: *const u8,
    out_content_hash: *mut u8,
    out_access_count: *mut u64,
) -> c_int {
    if handle.is_null() || node_id.is_null() || out_content_hash.is_null() || out_access_count.is_null() {
        return -1;
    }

    let handle = unsafe { &*handle };
    let graph = match handle.graph.lock() {
        Ok(g) => g,
        Err(_) => return -2,
    };

    let node_id_slice = unsafe { slice::from_raw_parts(node_id, 32) };
    let mut node_id_arr = [0u8; 32];
    node_id_arr.copy_from_slice(node_id_slice);

    match graph.get_node(&node_id_arr) {
        Some(node) => {
            unsafe {
                ptr::copy_nonoverlapping(node.content_hash.as_ptr(), out_content_hash, 32);
                *out_access_count = node.access_count;
            }
            0
        }
        None => -4,
    }
}

/// Query semantica
#[no_mangle]
pub extern "C" fn wormgraph_semantic_query(
    handle: *mut WormGraphHandle,
    query_embedding: *const c_double,
    embedding_dim: usize,
    top_k: usize,
    min_similarity: c_double,
    out_results: *mut u8,
    out_count: *mut usize,
) -> c_int {
    if handle.is_null() || query_embedding.is_null() || out_results.is_null() || out_count.is_null() {
        return -1;
    }

    let handle = unsafe { &*handle };
    let graph = match handle.graph.lock() {
        Ok(g) => g,
        Err(_) => return -2,
    };

    let query_slice = unsafe { slice::from_raw_parts(query_embedding, embedding_dim) };
    let query: Vec<f32> = query_slice.iter().map(|&x| x as f32).collect();

    match graph.semantic_query(&query, top_k, min_similarity as f32) {
        Ok(results) => {
            let count = results.len().min(top_k);
            unsafe {
                *out_count = count;
                // out_results deve ser um buffer de (32 bytes NodeId + 4 bytes similarity) * top_k
                for (i, (node, sim)) in results.iter().take(count).enumerate() {
                    let offset = i * 36;
                    ptr::copy_nonoverlapping(node.id.as_ptr(), out_results.add(offset), 32);
                    let sim_bytes = sim.to_le_bytes();
                    ptr::copy_nonoverlapping(sim_bytes.as_ptr(), out_results.add(offset + 32), 4);
                }
            }
            0
        }
        Err(_) => -5,
    }
}

/// Gera ZK nullifier
#[no_mangle]
pub extern "C" fn wormgraph_generate_nullifier(
    handle: *mut WormGraphHandle,
    node_id: *const u8,
    query_intent: *const c_char,
    out_nullifier: *mut u8,
) -> c_int {
    if handle.is_null() || node_id.is_null() || query_intent.is_null() || out_nullifier.is_null() {
        return -1;
    }

    let handle = unsafe { &*handle };
    let graph = match handle.graph.lock() {
        Ok(g) => g,
        Err(_) => return -2,
    };

    let node_id_slice = unsafe { slice::from_raw_parts(node_id, 32) };
    let mut node_id_arr = [0u8; 32];
    node_id_arr.copy_from_slice(node_id_slice);

    let query = unsafe { CStr::from_ptr(query_intent).to_string_lossy().into_owned() };

    match graph.generate_zk_nullifier(&node_id_arr, &query) {
        Ok(nullifier) => {
            unsafe {
                ptr::copy_nonoverlapping(nullifier.as_ptr(), out_nullifier, 32);
            }
            0
        }
        Err(_) => -6,
    }
}

/// Exporta snapshot FAIR
#[no_mangle]
pub extern "C" fn wormgraph_export_fair(
    handle: *mut WormGraphHandle,
    out_json: *mut c_char,
    max_len: usize,
) -> c_int {
    if handle.is_null() || out_json.is_null() {
        return -1;
    }

    let handle = unsafe { &*handle };
    let graph = match handle.graph.lock() {
        Ok(g) => g,
        Err(_) => return -2,
    };

    let snapshot = graph.export_fair_snapshot();
    let json = serde_json::to_string(&snapshot).unwrap_or_default();

    let c_json = CString::new(json).unwrap_or_default();
    let bytes = c_json.as_bytes_with_nul();
    let len = bytes.len().min(max_len);

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_json as *mut u8, len);
    }

    0
}

/// Calcula Phi_C
#[no_mangle]
pub extern "C" fn wormgraph_compute_phi_c(handle: *mut WormGraphHandle) -> c_double {
    if handle.is_null() {
        return 0.0;
    }

    let handle = unsafe { &*handle };
    let graph = match handle.graph.lock() {
        Ok(g) => g,
        Err(_) => return 0.0,
    };

    graph.compute_phi_c()
}

// =============================================================================
// PyO3 Bindings - Python nativo
// =============================================================================

#[cfg(feature = "pyo3")]
mod pyo3_bindings {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList, PyBytes};
    use super::*;

    #[pyclass]
    struct PyWormGraph {
        graph: WormGraph,
    }

    #[pymethods]
    impl PyWormGraph {
        #[new]
        fn new(max_tokens: usize, embedding_dim: Option<usize>) -> Self {
            let mut graph = WormGraph::new(max_tokens);
            if let Some(dim) = embedding_dim {
                graph = graph.with_embedding_dim(dim);
            }
            Self { graph }
        }

        fn add_node(&mut self, content: &str, substrate_id: &str, tags: Vec<String>,
                   dna: Vec<String>, embedding: Vec<f32>) -> PyResult<PyObject> {
            let dna_set: std::collections::BTreeSet<FoundingFather> = dna.iter()
                .filter_map(|s: &String| match s.as_str() {
                    "aristoteles" => Some(FoundingFather::Aristoteles),
                    "al_khwarizmi" => Some(FoundingFather::AlKhwarizmi),
                    "hipparchus" => Some(FoundingFather::Hipparchus),
                    "hippocrates" => Some(FoundingFather::Hippocrates),
                    "pasteur" => Some(FoundingFather::Pasteur),
                    "mendel" => Some(FoundingFather::Mendel),
                    "adam_smith" => Some(FoundingFather::AdamSmith),
                    "ada_lovelace" => Some(FoundingFather::AdaLovelace),
                    "vint_cerf" => Some(FoundingFather::VintCerf),
                    "einstein" => Some(FoundingFather::Einstein),
                    "feynman" => Some(FoundingFather::Feynman),
                    "rohrer" => Some(FoundingFather::Rohrer),
                    _ => None,
                })
                .collect();

            let metadata = NodeMetadata {
                substrate_id: substrate_id.to_string(),
                phi_c: 0.99,
                theosis: 0.98,
                tags,
                cross_links: vec![],
                version: String::from("5.2.0"),
                seal: [0u8; 32],
                timestamp_ns: 1_000_000_000,
            };

            match self.graph.add_node(content, metadata, dna_set, embedding) {
                Ok(node_id) => {
                    Python::with_gil(|py| {
                        Ok(PyBytes::new(py, &node_id).into())
                    })
                }
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e))),
            }
        }

        fn semantic_query(&self, query_embedding: Vec<f32>, top_k: usize, min_similarity: f32) -> PyResult<PyObject> {
            match self.graph.semantic_query(&query_embedding, top_k, min_similarity) {
                Ok(results) => {
                    Python::with_gil(|py| {
                        let list = PyList::empty(py);
                        for (node, sim) in results {
                            let dict = PyDict::new(py);
                            dict.set_item("id", PyBytes::new(py, &node.id))?;
                            dict.set_item("similarity", sim)?;
                            dict.set_item("access_count", node.access_count)?;
                            list.append(dict)?;
                        }
                        Ok(list.into())
                    })
                }
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e))),
            }
        }

        fn compute_phi_c(&self) -> f64 {
            self.graph.compute_phi_c()
        }

        fn export_fair(&self) -> PyResult<String> {
            let snapshot = self.graph.export_fair_snapshot();
            serde_json::to_string(&snapshot)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        }
    }

    #[pymodule]
    fn wormgraph(_py: Python, m: &PyModule) -> PyResult<()> {
        m.add_class::<PyWormGraph>()?;
        Ok(())
    }
}
