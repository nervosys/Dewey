//! Compute shader support — general-purpose GPU compute through the ontology.
//!
//! Provides [`ComputeTask`] for defining GPU compute operations with typed
//! input/output buffers, and [`ComputeDispatch`] for execution parameters.

use crate::ontology::{
    AgentAction, AgentCapability, Discoverable, SemanticRole, UiNode, WidgetSchema,
};

/// Describes a compute dispatch invocation.
#[derive(Debug, Clone, Copy)]
pub struct ComputeDispatch {
    pub workgroups_x: u32,
    pub workgroups_y: u32,
    pub workgroups_z: u32,
}

impl ComputeDispatch {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            workgroups_x: x,
            workgroups_y: y,
            workgroups_z: z,
        }
    }

    pub fn linear(count: u32, workgroup_size: u32) -> Self {
        let groups = count.div_ceil(workgroup_size);
        Self::new(groups, 1, 1)
    }

    pub fn grid_2d(width: u32, height: u32, workgroup_size: u32) -> Self {
        let gx = width.div_ceil(workgroup_size);
        let gy = height.div_ceil(workgroup_size);
        Self::new(gx, gy, 1)
    }

    pub fn total_invocations(&self, workgroup_size: [u32; 3]) -> u64 {
        self.workgroups_x as u64 * workgroup_size[0] as u64
            * self.workgroups_y as u64 * workgroup_size[1] as u64
            * self.workgroups_z as u64 * workgroup_size[2] as u64
    }
}

/// Describes a buffer binding for a compute task.
#[derive(Debug, Clone)]
pub struct ComputeBinding {
    pub name: String,
    pub group: u32,
    pub binding: u32,
    pub read_only: bool,
}

impl ComputeBinding {
    pub fn storage(name: impl Into<String>, group: u32, binding: u32) -> Self {
        Self {
            name: name.into(),
            group,
            binding,
            read_only: false,
        }
    }

    pub fn read_only(name: impl Into<String>, group: u32, binding: u32) -> Self {
        Self {
            name: name.into(),
            group,
            binding,
            read_only: true,
        }
    }
}

/// A compute task definition with shader source, bindings, and workgroup size.
pub struct ComputeTask {
    id: String,
    label: String,
    shader_source: String,
    entry_point: String,
    workgroup_size: [u32; 3],
    bindings: Vec<ComputeBinding>,
}

impl ComputeTask {
    pub fn new(id: impl Into<String>, label: impl Into<String>, shader_source: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shader_source: shader_source.into(),
            entry_point: "main".to_string(),
            workgroup_size: [64, 1, 1],
            bindings: Vec::new(),
        }
    }

    pub fn entry_point(mut self, entry: impl Into<String>) -> Self {
        self.entry_point = entry.into();
        self
    }

    pub fn workgroup_size(mut self, x: u32, y: u32, z: u32) -> Self {
        self.workgroup_size = [x, y, z];
        self
    }

    pub fn binding(mut self, binding: ComputeBinding) -> Self {
        self.bindings.push(binding);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn shader_source(&self) -> &str {
        &self.shader_source
    }

    pub fn entry(&self) -> &str {
        &self.entry_point
    }

    pub fn workgroup(&self) -> [u32; 3] {
        self.workgroup_size
    }

    pub fn bindings(&self) -> &[ComputeBinding] {
        &self.bindings
    }
}

impl Discoverable for ComputeTask {
    fn schema(&self) -> WidgetSchema {
        WidgetSchema::new("ComputeTask", "A GPU compute task", SemanticRole::System)
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::Custom("gpu_compute".into())]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::simple("dispatch", "Execute the compute task", true),
            AgentAction::simple("get_bindings", "List buffer bindings", false),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "label": self.label,
            "entry_point": self.entry_point,
            "workgroup_size": self.workgroup_size,
            "bindings": self.bindings.iter().map(|b| {
                serde_json::json!({
                    "name": b.name,
                    "group": b.group,
                    "binding": b.binding,
                    "read_only": b.read_only,
                })
            }).collect::<Vec<_>>(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "get_bindings" => Ok(serde_json::json!({
                "bindings": self.bindings.iter().map(|b| &b.name).collect::<Vec<_>>()
            })),
            "dispatch" => {
                // Actual GPU dispatch requires GpuContext; this is the ontology interface
                Ok(serde_json::json!({ "status": "requires_gpu_context" }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }

    fn agent_id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

impl ComputeTask {
    pub fn ui_node(&self) -> UiNode {
        UiNode::new("ComputeTask", SemanticRole::System).with_id(&self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_linear() {
        let d = ComputeDispatch::linear(1000, 64);
        assert_eq!(d.workgroups_x, 16); // ceil(1000/64) = 16
        assert_eq!(d.workgroups_y, 1);
    }

    #[test]
    fn dispatch_grid_2d() {
        let d = ComputeDispatch::grid_2d(256, 256, 16);
        assert_eq!(d.workgroups_x, 16);
        assert_eq!(d.workgroups_y, 16);
    }

    #[test]
    fn dispatch_total_invocations() {
        let d = ComputeDispatch::new(4, 4, 1);
        assert_eq!(d.total_invocations([8, 8, 1]), 4 * 8 * 4 * 8);
    }

    #[test]
    fn compute_binding_storage() {
        let b = ComputeBinding::storage("output", 0, 0);
        assert!(!b.read_only);
    }

    #[test]
    fn compute_binding_readonly() {
        let b = ComputeBinding::read_only("input", 0, 1);
        assert!(b.read_only);
    }

    #[test]
    fn compute_task_builder() {
        let task = ComputeTask::new("add_vectors", "Vector addition", "@compute fn main() {}")
            .entry_point("main")
            .workgroup_size(256, 1, 1)
            .binding(ComputeBinding::read_only("a", 0, 0))
            .binding(ComputeBinding::read_only("b", 0, 1))
            .binding(ComputeBinding::storage("result", 0, 2));
        assert_eq!(task.bindings().len(), 3);
        assert_eq!(task.workgroup(), [256, 1, 1]);
    }

    #[test]
    fn compute_task_discoverable() {
        let task = ComputeTask::new("t1", "Test", "");
        assert_eq!(task.semantic_role(), SemanticRole::System);
        let state = task.agent_state();
        assert_eq!(state["id"], "t1");
    }

    #[test]
    fn compute_task_execute_get_bindings() {
        let mut task = ComputeTask::new("t1", "Test", "")
            .binding(ComputeBinding::storage("out", 0, 0));
        let result = task.execute_action("get_bindings", &serde_json::json!({}));
        assert!(result.is_ok());
    }

    #[test]
    fn compute_task_unknown_action() {
        let mut task = ComputeTask::new("t1", "Test", "");
        let result = task.execute_action("nope", &serde_json::json!({}));
        assert!(result.is_err());
    }
}
