use crate::{
    command::Command, define_node_command, define_swap_command, define_vec_add_remove_commands,
    scene::commands::SceneContext,
};
use fyrox::{
    animation::Animation,
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::{Handle, Ticket},
        visitor::Visitor,
    },
    scene::{
        base::{deserialize_script, visit_opt_script, Mobility, Property, PropertyValue},
        graph::{Graph, SubGraph},
        node::Node,
    },
    script::Script,
};
use std::io::Cursor;

#[derive(Debug)]
pub struct MoveNodeCommand {
    node: Handle<Node>,
    old_position: Vector3<f32>,
    new_position: Vector3<f32>,
}

impl MoveNodeCommand {
    pub fn new(node: Handle<Node>, old_position: Vector3<f32>, new_position: Vector3<f32>) -> Self {
        Self {
            node,
            old_position,
            new_position,
        }
    }

    fn swap(&mut self) -> Vector3<f32> {
        let position = self.new_position;
        std::mem::swap(&mut self.new_position, &mut self.old_position);
        position
    }

    fn set_position(&self, graph: &mut Graph, position: Vector3<f32>) {
        graph[self.node]
            .local_transform_mut()
            .set_position(position);
    }
}

impl Command for MoveNodeCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Move Node".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let position = self.swap();
        self.set_position(&mut context.scene.graph, position);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let position = self.swap();
        self.set_position(&mut context.scene.graph, position);
    }
}

#[derive(Debug)]
pub struct ScaleNodeCommand {
    node: Handle<Node>,
    old_scale: Vector3<f32>,
    new_scale: Vector3<f32>,
}

impl ScaleNodeCommand {
    pub fn new(node: Handle<Node>, old_scale: Vector3<f32>, new_scale: Vector3<f32>) -> Self {
        Self {
            node,
            old_scale,
            new_scale,
        }
    }

    fn swap(&mut self) -> Vector3<f32> {
        let position = self.new_scale;
        std::mem::swap(&mut self.new_scale, &mut self.old_scale);
        position
    }

    fn set_scale(&self, graph: &mut Graph, scale: Vector3<f32>) {
        graph[self.node].local_transform_mut().set_scale(scale);
    }
}

impl Command for ScaleNodeCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Scale Node".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let scale = self.swap();
        self.set_scale(&mut context.scene.graph, scale);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let scale = self.swap();
        self.set_scale(&mut context.scene.graph, scale);
    }
}

#[derive(Debug)]
pub struct RotateNodeCommand {
    node: Handle<Node>,
    old_rotation: UnitQuaternion<f32>,
    new_rotation: UnitQuaternion<f32>,
}

impl RotateNodeCommand {
    pub fn new(
        node: Handle<Node>,
        old_rotation: UnitQuaternion<f32>,
        new_rotation: UnitQuaternion<f32>,
    ) -> Self {
        Self {
            node,
            old_rotation,
            new_rotation,
        }
    }

    fn swap(&mut self) -> UnitQuaternion<f32> {
        let position = self.new_rotation;
        std::mem::swap(&mut self.new_rotation, &mut self.old_rotation);
        position
    }

    fn set_rotation(&self, graph: &mut Graph, rotation: UnitQuaternion<f32>) {
        graph[self.node]
            .local_transform_mut()
            .set_rotation(rotation);
    }
}

impl Command for RotateNodeCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Rotate Node".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let rotation = self.swap();
        self.set_rotation(&mut context.scene.graph, rotation);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let rotation = self.swap();
        self.set_rotation(&mut context.scene.graph, rotation);
    }
}

#[derive(Debug)]
pub struct LinkNodesCommand {
    child: Handle<Node>,
    parent: Handle<Node>,
}

impl LinkNodesCommand {
    pub fn new(child: Handle<Node>, parent: Handle<Node>) -> Self {
        Self { child, parent }
    }

    fn link(&mut self, graph: &mut Graph) {
        let old_parent = graph[self.child].parent();
        graph.link_nodes(self.child, self.parent);
        self.parent = old_parent;
    }
}

impl Command for LinkNodesCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Link Nodes".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        self.link(&mut context.scene.graph);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.link(&mut context.scene.graph);
    }
}

#[derive(Debug)]
pub struct DeleteNodeCommand {
    handle: Handle<Node>,
    ticket: Option<Ticket<Node>>,
    node: Option<Node>,
    parent: Handle<Node>,
}

impl Command for DeleteNodeCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Delete Node".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        self.parent = context.scene.graph[self.handle].parent();
        let (ticket, node) = context.scene.graph.take_reserve(self.handle);
        self.node = Some(node);
        self.ticket = Some(ticket);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.handle = context
            .scene
            .graph
            .put_back(self.ticket.take().unwrap(), self.node.take().unwrap());
        context.scene.graph.link_nodes(self.handle, self.parent);
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context
                .scene
                .graph
                .forget_ticket(ticket, self.node.take().unwrap());
        }
    }
}

#[derive(Debug)]
pub struct AddModelCommand {
    model: Handle<Node>,
    animations: Vec<Handle<Animation>>,
    sub_graph: Option<SubGraph>,
    animations_container: Vec<(Ticket<Animation>, Animation)>,
}

impl AddModelCommand {
    pub fn new(
        sub_graph: SubGraph,
        animations_container: Vec<(Ticket<Animation>, Animation)>,
    ) -> Self {
        Self {
            model: Default::default(),
            animations: Default::default(),
            sub_graph: Some(sub_graph),
            animations_container,
        }
    }
}

impl Command for AddModelCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Load Model".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        // A model was loaded, but change was reverted and here we must put all nodes
        // back to graph.
        self.model = context
            .scene
            .graph
            .put_sub_graph_back(self.sub_graph.take().unwrap());
        for (ticket, animation) in self.animations_container.drain(..) {
            context.scene.animations.put_back(ticket, animation);
        }
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.sub_graph = Some(context.scene.graph.take_reserve_sub_graph(self.model));
        self.animations_container = self
            .animations
            .iter()
            .map(|&anim| context.scene.animations.take_reserve(anim))
            .collect();
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(sub_graph) = self.sub_graph.take() {
            context.scene.graph.forget_sub_graph(sub_graph)
        }
        for (ticket, _) in self.animations_container.drain(..) {
            context.scene.animations.forget_ticket(ticket);
        }
    }
}

#[derive(Debug)]
pub struct DeleteSubGraphCommand {
    sub_graph_root: Handle<Node>,
    sub_graph: Option<SubGraph>,
    parent: Handle<Node>,
}

impl DeleteSubGraphCommand {
    pub fn new(sub_graph_root: Handle<Node>) -> Self {
        Self {
            sub_graph_root,
            sub_graph: None,
            parent: Handle::NONE,
        }
    }
}

impl Command for DeleteSubGraphCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Delete Sub Graph".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        self.parent = context.scene.graph[self.sub_graph_root].parent();
        self.sub_graph = Some(
            context
                .scene
                .graph
                .take_reserve_sub_graph(self.sub_graph_root),
        );
    }

    fn revert(&mut self, context: &mut SceneContext) {
        context
            .scene
            .graph
            .put_sub_graph_back(self.sub_graph.take().unwrap());
        context
            .scene
            .graph
            .link_nodes(self.sub_graph_root, self.parent);
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(sub_graph) = self.sub_graph.take() {
            context.scene.graph.forget_sub_graph(sub_graph)
        }
    }
}

#[derive(Debug)]
pub struct AddNodeCommand {
    ticket: Option<Ticket<Node>>,
    handle: Handle<Node>,
    node: Option<Node>,
    cached_name: String,
    parent: Handle<Node>,
}

impl AddNodeCommand {
    pub fn new(node: Node, parent: Handle<Node>) -> Self {
        Self {
            ticket: None,
            handle: Default::default(),
            cached_name: format!("Add Node {}", node.name()),
            node: Some(node),
            parent,
        }
    }
}

impl Command for AddNodeCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        self.cached_name.clone()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        match self.ticket.take() {
            None => {
                self.handle = context.scene.graph.add_node(self.node.take().unwrap());
            }
            Some(ticket) => {
                let handle = context
                    .scene
                    .graph
                    .put_back(ticket, self.node.take().unwrap());
                assert_eq!(handle, self.handle);
            }
        }

        context.scene.graph.link_nodes(self.handle, self.parent)
    }

    fn revert(&mut self, context: &mut SceneContext) {
        // No need to unlink node from its parent because .take_reserve() does that for us.
        let (ticket, node) = context.scene.graph.take_reserve(self.handle);
        self.ticket = Some(ticket);
        self.node = Some(node);
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context
                .scene
                .graph
                .forget_ticket(ticket, self.node.take().unwrap());
        }
    }
}

define_vec_add_remove_commands!(
    struct AddPropertyCommand, RemovePropertyCommand<Node, Property>
    (self, context) { context.scene.graph[self.handle].properties.get_mut() }
);

#[derive(Debug)]
pub struct SetPropertyValueCommand {
    pub handle: Handle<Node>,
    pub index: usize,
    pub value: PropertyValue,
}

impl SetPropertyValueCommand {
    fn swap(&mut self, context: &mut SceneContext) {
        std::mem::swap(
            &mut context.scene.graph[self.handle].properties.get_mut()[self.index].value,
            &mut self.value,
        );
    }
}

impl Command for SetPropertyValueCommand {
    fn name(&mut self, _: &SceneContext) -> String {
        "Set Property Value".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        self.swap(context)
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.swap(context)
    }
}

#[derive(Debug)]
pub struct SetPropertyNameCommand {
    pub handle: Handle<Node>,
    pub index: usize,
    pub name: String,
}

impl SetPropertyNameCommand {
    fn swap(&mut self, context: &mut SceneContext) {
        std::mem::swap(
            &mut context.scene.graph[self.handle].properties.get_mut()[self.index].name,
            &mut self.name,
        );
    }
}

impl Command for SetPropertyNameCommand {
    fn name(&mut self, _: &SceneContext) -> String {
        "Set Property Name".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        self.swap(context)
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.swap(context)
    }
}

fn node_mut(node: &mut Node) -> &mut Node {
    node
}

define_swap_command! {
    node_mut,
    SetNameCommand(String): name_owned, set_name, "Set Name";
    SetTagCommand(String): tag_owned, set_tag, "Set Tag";
    SetFrustumCullingCommand(bool): frustum_culling, set_frustum_culling, "Set Frustum Culling";
    SetVisibleCommand(bool): visibility, set_visibility, "Set Visible";
    //SetLifetimeCommand(Option<f32>): lifetime, set_lifetime, "Set Lifetime";
    SetMobilityCommand(Mobility): mobility, set_mobility, "Set Mobility";
    SetDepthOffsetCommand(f32): depth_offset_factor, set_depth_offset_factor, "Set Depth Offset";
    SetCastShadowsCommand(bool): cast_shadows, set_cast_shadows, "Set Cast Shadows";
}

define_node_command! {
    SetPostRotationCommand("Set Post Rotation", UnitQuaternion<f32>) where fn swap(self, node) {
        let temp = **node.local_transform().post_rotation();
        node.local_transform_mut().set_post_rotation(self.value);
        self.value = temp;
    }

    SetPreRotationCommand("Set Pre Rotation", UnitQuaternion<f32>) where fn swap(self, node) {
        let temp = **node.local_transform().pre_rotation();
        node.local_transform_mut().set_pre_rotation(self.value);
        self.value = temp;
    }

    SetRotationOffsetCommand("Set Rotation Offset", Vector3<f32>) where fn swap(self, node) {
        let temp = **node.local_transform().rotation_offset();
        node.local_transform_mut().set_rotation_offset(self.value);
        self.value = temp;
    }

    SetRotationPivotCommand("Set Rotation Pivot", Vector3<f32>) where fn swap(self, node) {
        let temp = **node.local_transform().rotation_pivot();
        node.local_transform_mut().set_rotation_pivot(self.value);
        self.value = temp;
    }

    SetScaleOffsetCommand("Set Scaling Offset", Vector3<f32>) where fn swap(self, node) {
        let temp = **node.local_transform().scaling_offset();
        node.local_transform_mut().set_scaling_offset(self.value);
        self.value = temp;
    }

    SetScalePivotCommand("Set Scaling Pivot", Vector3<f32>) where fn swap(self, node) {
        let temp = **node.local_transform().scaling_pivot();
        node.local_transform_mut().set_scaling_pivot(self.value);
        self.value = temp;
    }
}

#[derive(Debug)]
pub enum SetScriptCommandState {
    Undefined,
    NonExecuted { script: Option<Script> },
    Executed,
    Reverted { data: Vec<u8> },
}

#[derive(Debug)]
pub struct SetScriptCommand {
    handle: Handle<Node>,
    state: SetScriptCommandState,
}

impl SetScriptCommand {
    pub fn new(handle: Handle<Node>, script: Option<Script>) -> Self {
        Self {
            handle,
            state: SetScriptCommandState::NonExecuted { script },
        }
    }
}

impl Command for SetScriptCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Set Script Command".to_string()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let node = &mut context.scene.graph[self.handle];

        match std::mem::replace(&mut self.state, SetScriptCommandState::Executed) {
            SetScriptCommandState::NonExecuted { script } => {
                node.script = script;
            }
            SetScriptCommandState::Reverted { data } => {
                let mut visitor = Visitor::load_from_memory(data).unwrap();
                visitor.environment = Some(context.serialization_context.clone());
                visit_opt_script("Script", &mut node.script, &mut visitor).unwrap();
            }
            _ => unreachable!(),
        }
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let node = &mut context.scene.graph[self.handle];
        match std::mem::replace(&mut self.state, SetScriptCommandState::Undefined) {
            SetScriptCommandState::Executed => {
                let mut script = node.script.take();
                let mut visitor = Visitor::new();
                visitor.environment = Some(context.serialization_context.clone());
                visit_opt_script("Script", &mut script, &mut visitor).unwrap();
                let mut data = Cursor::new(Vec::<u8>::new());
                visitor.save_binary_to_memory(&mut data).unwrap();
                self.state = SetScriptCommandState::Reverted {
                    data: data.into_inner(),
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct ScriptDataBlobCommand {
    pub handle: Handle<Node>,
    pub old_value: Vec<u8>,
    pub new_value: Vec<u8>,
}

impl ScriptDataBlobCommand {
    fn swap(&mut self, context: &mut SceneContext) {
        let data = self.new_value.clone();
        std::mem::swap(&mut self.old_value, &mut self.new_value);
        if let Some(script) = context.scene.graph[self.handle].script.as_mut() {
            *script = deserialize_script(data, &context.serialization_context).unwrap();
            script.restore_resources(context.resource_manager.clone());
        } else {
            unreachable!()
        }
    }
}

impl Command for ScriptDataBlobCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Change Script Property".to_string()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        self.swap(context);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.swap(context);
    }
}
