# AGENTS.md: マルチエージェントオーケストレーション実装仕様

## 1. システム概要

本ドキュメントは、Luminara EngineにおけるAIエージェントの協調システム（Agent Orchestrator）のアーキテクチャおよび実装仕様を定義する。単一のLLMによるシーケンシャルな処理の限界を突破し、複数の特化型エージェントが並行してゲームコンテンツを構築・検証する環境を提供する。

## 2. エージェントの役割と権限 (Roles & Permissions)

システムに登録される各エージェントは、厳密に定義された役割（Role）と権限（Permissions）を持つ。これにより、不正なリソースアクセスを防ぎ、安全なタスク分割を実現する。

```rust
bitflags::bitflags! {
    pub struct AgentPermissions: u32 {
        const READ_SCENE       = 0b0000_0001;
        const WRITE_SCENE      = 0b0000_0010;
        const READ_SCRIPT      = 0b0000_0100;
        const WRITE_SCRIPT     = 0b0000_1000;
        const EXECUTE_CODE     = 0b0001_0000;
        const MANAGE_TASKS     = 0b0010_0000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentRole {
    ProjectDirector,
    SceneArchitect,
    GameplayProgrammer,
    ArtDirector,
    QAEngineer,
}
//

```

### 役割定義

* **ProjectDirector**: ユーザーの自然言語リクエストを解析し、サブタスクへの分解と実行順序（依存関係）のグラフ構築を行う。(`MANAGE_TASKS` 権限)
* **SceneArchitect**: エンティティのスポーン、空間配置、環境構築など、ECSの世界の構造に責任を持つ。(`READ_SCENE`, `WRITE_SCENE` 権限)
* **GameplayProgrammer**: Lua/WASMスクリプトの生成、ゲームロジックの実装、コンポーネントの操作を行う。(`READ_SCRIPT`, `WRITE_SCRIPT`, `EXECUTE_CODE`, `WRITE_SCENE` 権限)
* **ArtDirector**: Visual Feedback Systemと連携し、レンダリング結果の検証や、アセットの整合性確認、ライティングの調整を行う。(`READ_SCENE`, `WRITE_SCENE` 権限)
* **QAEngineer**: Code Verifier（静的解析・サンドボックス実行）と連携し、生成されたロジックの安全性とパフォーマンス要件を満たしているかテストする。(`READ_SCRIPT`, `EXECUTE_CODE` 権限)

## 3. タスクオーケストレーション (Task Decomposition & Execution)

ProjectDirectorによるタスク分解後、`AgentOrchestrator` は有向非巡回グラフ (DAG) を構築し、タスクをスケジューリングする。

1. **解析と分解**: ユーザーリクエストを独立したサブタスクに分割。
2. **依存関係グラフの構築**: タスクBがタスクAの結果を必要とする場合、直列化する。
3. **並列実行**: 依存関係のないタスクは、`tokio::spawn` を用いて並行ワーカープールで即座に非同期実行される。

```rust
pub struct TaskGraph {
    nodes: HashMap<TaskId, SubTask>,
    edges: HashMap<TaskId, Vec<TaskId>>, // 依存関係
}

impl AgentOrchestrator {
    /// 独立したタスクを並列実行し、依存タスクを順次解決する
    pub async fn execute_graph(&mut self, graph: TaskGraph) -> Result<OrchestrationResult> {
        // 実装: トポロジカルソートと tokio::task::JoinSet を用いた並列ワーカー制御
        //
    }
}

```

## 4. コンフリクト検知と解決 (Conflict Detection & Resolution)

複数のエージェントが同時に同じエンティティやファイルを操作した場合の競合を防ぐ。ECSの特性を活かし、**エンティティ単位ではなく、コンポーネント単位のロック/差分検知**を行うことで並列性を最大化する。

* **競合なし**: エージェントAがEntity Xの `Transform` を変更し、エージェントBがEntity Xの `Physics` を変更した場合。
* **競合あり**: 両者がEntity Xの `Transform` を変更した場合。

```rust
pub enum ResolutionStrategy {
    LastWriteWins,
    Merge(MergeStrategy),
    PromptUser, //
}

impl Orchestrator {
    /// 意図（Intent）の適用前にコンフリクトを検知する
    pub fn detect_conflicts(intents: &[AiIntent]) -> Vec<Conflict> {
        // 対象のEntity IDとComponent TypeIdのハッシュセットで交差を判定
        //
    }
}

```

## 5. エージェント間メッセージバス (Inter-Agent Messaging)

エージェントが協調して動作するため、非同期のパブリッシュ/サブスクライブ型メッセージバスを実装する。これは、高いスループットと低レイテンシが求められるルーティングプロトコルと同様の堅牢な非同期設計を採用する。

```rust
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub sender: AgentRole,
    pub recipient: AgentRole,
    pub payload: MessagePayload,
    pub timestamp: std::time::Instant,
}

pub struct MessageBus {
    sender: tokio::sync::broadcast::Sender<AgentMessage>,
}

impl MessageBus {
    /// 1オーケストレーションサイクル内でメッセージを確実に配信する
    ///
    pub async fn publish(&self, msg: AgentMessage) -> Result<()> {
        self.sender.send(msg).map_err(Into::into)
    }
}

```

## 6. 変更の集約とサマリー生成 (Change Summarization)

全エージェントのタスクが完了した後、OrchestratorはOperation Timelineと連携し、実行された全操作をロールごとに構造化してユーザーに提示する。

* **構造化サマリー**: どエージェントが、どのエンティティ/スクリプトに対して、どのような変更を加えたかのDiffツリー。
* **パフォーマンスレポート**: Performance Advisorが予測したFPS/メモリへの影響評価。

---

