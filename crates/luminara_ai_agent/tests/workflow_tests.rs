use luminara_ai_agent::{AiContextEngine, IntentResolver, CodeVerificationPipeline};
use luminara_core::world::World;
use luminara_script_lua::LuaScriptRuntime;
use std::sync::Arc;

#[test]
fn test_end_to_end_ai_workflow() {
    let mut world = World::new();
    let mut context_engine = AiContextEngine::new();
    let semantic_index = Arc::new(luminara_ai_agent::SemanticIndex::new());
    let resolver = IntentResolver::new(semantic_index.clone());
    let mut verifier = CodeVerificationPipeline::new().unwrap();

    let ctx = context_engine.generate_context("Spawn a cube", 1000, &world);
    assert!(!ctx.summary.is_empty());

    let generated_code = "return { on_start = function() print('Spawned') end }";

    let (result, _) = verifier.verify_and_apply(generated_code, &mut world);

    assert!(result.passed);
    assert!(result.sandbox_result.unwrap().is_ok());
}
