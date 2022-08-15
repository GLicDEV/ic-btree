mod env;
mod lifetime;

use crate::env::{CanisterEnv, EmptyEnv, Environment};
use candid::{candid_method, CandidType};
use ic_cdk_macros::*;
use serde::Deserialize;

use std::cell::RefCell;

use std::collections::BTreeMap;

thread_local! {
    static RUNTIME_STATE: RefCell<RuntimeState> = RefCell::default();
}

struct RuntimeState {
    pub env: Box<dyn Environment>,
    pub data: Data,
}

impl Default for RuntimeState {
    fn default() -> Self {
        RuntimeState {
            env: Box::new(EmptyEnv {}),
            data: Data::default(),
        }
    }
}

#[derive(CandidType, Default, Deserialize)]
struct Data {
    items: Vec<String>,
    btree: BTreeMap<String, u64>,
}

//getStableStateSize
#[candid_method(query, rename = "getStableStateSize")]
#[query(name = "getStableStateSize")]
fn state_size() -> u64 {
    RUNTIME_STATE.with(|state| state.borrow().env.memory_used())
}

//getBTreeLength
#[candid_method(query, rename = "getBTreeLength")]
#[query(name = "getBTreeLength")]
fn tree_len() -> usize {
    RUNTIME_STATE.with(|state| state.borrow().data.btree.len())
}

//getBTreeItem
#[candid_method(query, rename = "getBTreeItem")]
#[query(name = "getBTreeItem")]
fn get_btree_item(k: String) -> Option<u64> {
    RUNTIME_STATE.with(|state| get_btree_item_impl(k, &mut state.borrow_mut()))
}

fn get_btree_item_impl(
    k: String,
    runtime_state: &mut std::cell::RefMut<RuntimeState>,
) -> Option<u64> {
    runtime_state.data.btree.get(&k).cloned()
}

//allocateRBTSpace
#[candid_method(update, rename = "allocateRBTSpace")]
#[update(name = "allocateRBTSpace")]
fn add_btree(start: u64, count: u64) -> u64 {
    RUNTIME_STATE.with(|state| add_btree_impl(start, count, &mut state.borrow_mut()))
}

fn add_btree_impl(start: u64, count: u64, runtime_state: &mut RuntimeState) -> u64 {
    let end = start + count;

    for i in start..end {
        runtime_state.data.btree.insert(format!("{}", i), i);
    }

    end
}

//growRBTSpace
#[candid_method(update, rename = "growRBTSpace")]
#[update(name = "growRBTSpace")]
fn grow_btree(count: u64) -> u64 {
    RUNTIME_STATE.with(|state| grow_btree_impl(count, &mut state.borrow_mut()))
}

fn grow_btree_impl(count: u64, runtime_state: &mut RuntimeState) -> u64 {
    let start = runtime_state.data.btree.len() as u64;
    let end = start + count;

    for i in start..end {
        runtime_state.data.btree.insert(format!("{}", i), i);
    }

    end
}

#[candid_method(query)]
#[query]
fn greet(name: String) -> String {
    format!("Hello, {}! Welcome!", name)
}

// Auto export the candid interface
candid::export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
#[candid_method(query, rename = "__get_candid_interface_tmp_hack")]
fn __export_did_tmp_() -> String {
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_candid() {
        let expected = String::from_utf8(std::fs::read("ic_btree.did").unwrap()).unwrap();

        let actual = __export_service();

        if actual != expected {
            println!("{}", actual);
        }

        assert_eq!(
            actual, expected,
            "Generated candid definition does not match expected did file"
        );
    }
}
