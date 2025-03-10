//! Animation blending state machine resource.

use crate::{
    animation::machine::{Machine, MachineDefinition, MachineInstantiationError},
    asset::{define_new_resource, Resource, ResourceData},
    core::{pool::Handle, visitor::prelude::*},
    engine::resource_manager::{options::ImportOptions, ResourceManager},
    scene::{node::Node, Scene},
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

/// State of the [`AbsmResource`]
#[derive(Debug, Visit, Default)]
pub struct AbsmResourceState {
    /// A path to source.
    pub path: PathBuf,
    /// Animation blending state machine definition.
    pub absm_definition: MachineDefinition,
}

impl ResourceData for AbsmResourceState {
    fn path(&self) -> Cow<Path> {
        Cow::Borrowed(&self.path)
    }

    fn set_path(&mut self, path: PathBuf) {
        self.path = path;
    }
}

impl AbsmResourceState {
    /// Load a ABSM resource from the specific file path.
    pub async fn from_file(path: &Path) -> Result<Self, MachineInstantiationError> {
        let mut visitor = Visitor::load_binary(path).await?;
        let mut absm_definition = MachineDefinition::default();
        absm_definition.visit("Machine", &mut visitor)?;
        Ok(Self {
            absm_definition,
            path: path.to_path_buf(),
        })
    }
}

define_new_resource!(
    /// See module docs.
    AbsmResource<AbsmResourceState, MachineInstantiationError>
);

impl AbsmResource {
    /// Instantiates animation blending state machine to the specified scene for a given root node.
    ///
    /// # Steps
    ///
    /// Instantiation involves multiple steps, the most important are:
    ///
    /// - Animation retargeting - it tries to retarget animation stored in PlayAnimation nodes to
    ///   a node hierarchy that starts from `root` node. The step may fail if the animation is not
    ///   suitable for the hierarchy.
    /// - ABSM instantiation - it uses ABSM definition to create a new instance of the ABSM.
    ///
    /// # Important notes
    ///
    /// The method loads multiple animation resources at once and it will fail even if one of them
    /// is faulty. Animation retargeting creates multiple animation instances in the scene, you
    /// **must** delete them manually when deleting the ABSM instance.
    ///
    /// The method is intended to be used with the ABSM resources made in the Fyroxed, any
    /// "hand-crafted" resources may contain invalid data which may cause errors during instantiation
    /// or even panic.  
    pub async fn instantiate(
        &self,
        root: Handle<Node>,
        scene: &mut Scene,
        resource_manager: ResourceManager,
    ) -> Result<Handle<Machine>, MachineInstantiationError> {
        let data = self.data_ref();
        let definition = &data.absm_definition;

        let machine = definition
            .instantiate(root, scene, resource_manager)
            .await?;

        scene.animation_machines[machine].resource = Some(self.clone());

        Ok(machine)
    }
}

/// Import options for ABSM resource.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AbsmImportOptions {}

impl ImportOptions for AbsmImportOptions {}
