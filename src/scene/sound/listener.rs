#![allow(missing_docs)] // TODO

use crate::{
    core::{
        inspect::{Inspect, PropertyInfo},
        pool::Handle,
        visitor::prelude::*,
    },
    scene::{
        base::{Base, BaseBuilder},
        graph::Graph,
        node::Node,
    },
};
use std::ops::{Deref, DerefMut};

#[derive(Visit, Inspect, Default, Debug)]
pub struct Listener {
    base: Base,
}

impl Deref for Listener {
    type Target = Base;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for Listener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Listener {
    pub fn raw_copy(&self) -> Self {
        Self {
            base: self.base.raw_copy(),
        }
    }

    // Prefab inheritance resolving.
    pub(crate) fn inherit(&mut self, parent: &Node) {
        self.base.inherit_properties(parent);
    }
}

pub struct ListenerBuilder {
    base_builder: BaseBuilder,
}

impl ListenerBuilder {
    pub fn new(base_builder: BaseBuilder) -> Self {
        Self { base_builder }
    }

    pub fn build_node(self) -> Node {
        Node::Listener(Listener {
            base: self.base_builder.build_base(),
        })
    }

    pub fn build(self, graph: &mut Graph) -> Handle<Node> {
        graph.add_node(self.build_node())
    }
}