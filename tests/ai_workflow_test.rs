use luminara_ai_agent::{AiContextEngine, IntentResolver, CodeVerificationPipeline};
use luminara_core::world::World;
use luminara_script_lua::LuaScriptRuntime;
use std::sync::Arc;

// End-to-end integration test flow:
// 1. User/AI generates intent (semantic search -> intent resolver).
// 2. Intent resolved to code/commands.
// 3. Code verified by pipeline.
// 4. Code executed in sandbox/engine.

#[test]
fn test_end_to_end_ai_workflow() {
    let mut world = World::new();
    let mut context_engine = AiContextEngine::new();
    let semantic_index = Arc::new(luminara_ai_agent::SemanticIndex::new());
    // Assume intent resolver uses same index
    let resolver = IntentResolver::new(semantic_index.clone());
    let mut verifier = CodeVerificationPipeline::new().unwrap();

    // 1. Context Generation
    let ctx = context_engine.generate_context("Spawn a cube", 1000, &world);
    assert!(!ctx.summary.is_empty());

    // 2. Resolve Intent (Mocked: "Spawn a cube" -> SpawnRelative)
    // Resolver `resolve` returns EngineCommand.
    // But AI might generate CODE.
    // If AI generates code:
    let generated_code = "return { on_start = function() print('Spawned') end }";

    // 3. Verify Code
    let (result, _) = verifier.verify_and_apply(generated_code, &mut world);

    // Since verify_and_apply attempts apply, and our applicator is mock, it succeeds if static/sandbox pass.
    assert!(result.passed);
    assert!(result.sandbox_result.unwrap().is_ok());

    // 4. Runtime check (Lua)
    // Code applicator (mock) doesn't actually run Lua in engine yet unless we wire it up.
    // But `sandbox.run_lua` executed it.

    // Check timeline?
    // We didn't integrate timeline here explicitly, but usually AgentOrchestrator manages that.
}
