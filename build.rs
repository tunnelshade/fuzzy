use prost_build::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();
    // The best magic to tie Protobuf Structs -> Diesel ORM Rust Structs
    // NewWorker (Worker with limited fields)
    config
        .type_attribute(
            "NewWorker",
            "#[derive(Queryable, Insertable, AsChangeset, Associations, Serialize, Deserialize)]",
        )
        .type_attribute("NewWorker", "#[table_name = \"workers\"]")
        // Worker
        .type_attribute("Worker", "#[derive(Queryable, Identifiable, Associations)]")
        .type_attribute("Worker", "#[table_name = \"workers\"]")
        // NewTask (Task with limited fields)
        .type_attribute("NewTask", "#[derive(Queryable, Insertable, AsChangeset, Associations)]")
        .type_attribute("NewTask", "#[table_name = \"tasks\"]")
        // PatchTask (Task with limited fields)
        .type_attribute("PatchTask", "#[derive(Queryable, AsChangeset, Associations)]")
        .type_attribute("PatchTask", "#[table_name = \"tasks\"]")
        // Task (Removed as prost_types::Timestamp cannto be changed)
        .type_attribute("Task", "#[derive(Queryable, Identifiable, Associations)]")
        .type_attribute("Task", "#[table_name = \"tasks\"]")
        // NewCorpus (Corpus with limited fields)
        .type_attribute(
            "NewCorpus",
            "#[derive(Queryable, Insertable, AsChangeset, Associations)]",
        )
        .type_attribute("NewCorpus", "#[table_name = \"corpora\"]")
        // Corpus
        .type_attribute("Corpus", "#[derive(Queryable, Identifiable, Associations)]")
        .type_attribute("Corpus", "#[table_name = \"corpora\"]")
        // NewCrash (Crash with limited fields)
        .type_attribute(
            "NewCrash",
            "#[derive(Queryable, Insertable, AsChangeset, Associations)]",
        )
        .type_attribute("NewCrash", "#[table_name = \"crashes\"]")
        // PatchCrash (Crash with limited fields)
        .type_attribute(
            "PatchCrash",
            "#[derive(Queryable, Insertable, AsChangeset, Associations)]",
        )
        .type_attribute("PatchCrash", "#[table_name = \"crashes\"]")
        .type_attribute("PatchCrash", "#[changeset_options(treat_none_as_null=\"true\")]")
        // Crash
        .type_attribute("Crash", "#[derive(Queryable, Identifiable, Associations)]")
        .type_attribute("Crash", "#[table_name = \"crashes\"]")
        // WorkerTask (Worker Task)
        .type_attribute("WorkerTask", "#[derive(Queryable, Identifiable, Associations)]")
        .type_attribute("WorkerTask", "#[table_name = \"worker_tasks\"]")
        .type_attribute("WorkerTask", "#[belongs_to(Task)]")
        .type_attribute("WorkerTask", "#[belongs_to(Worker)]")
        // PatchWorkerTask (Patch Worker Task to update state)
        .type_attribute("PatchWorkerTask", "#[derive(Queryable, AsChangeset, Associations)]")
        .type_attribute("PatchWorkerTask", "#[table_name = \"worker_tasks\"]")
        // WorkerTask (Worker Task)
        .type_attribute("WorkerTaskFull", "#[derive(Queryable, Associations)]")
        // .type_attribute("WorkerTaskFull", "#[table_name = \"worker_tasks\"]")
        // NewFuzzStat (FuzzStat without time field)
        .type_attribute(
            "NewFuzzStat",
            "#[derive(Queryable, Insertable, AsChangeset, Associations)]",
        )
        .type_attribute("NewFuzzStat", "#[table_name = \"fuzz_stats\"]")
        .type_attribute("NewFuzzStat", "#[belongs_to(WorkerTask)]")
        // NewSysStat (SysStat without time field)
        .type_attribute(
            "NewSysStat",
            "#[derive(Queryable, Insertable, AsChangeset, Associations)]",
        )
        .type_attribute("NewSysStat", "#[table_name = \"sys_stats\"]")
        .type_attribute("NewSysStat", "#[belongs_to(Worker)]")
        // NewTraceEvent
        .type_attribute(
            "NewTraceEvent",
            "#[derive(Queryable, Insertable, Associations)]",
        )
        .type_attribute("NewTraceEvent", "#[table_name = \"trace_events\"]")
        .type_attribute("NewTraceEvent", "#[belongs_to(Worker)]")
        // All fields of this name, prost converts them to prost_types::Timestamp, which diesel
        // doesn't support natively so we customize deserialization behaviour for one field
        //
        // https://docs.rs/diesel/1.4.4/diesel/deserialize/trait.Queryable.html
        //
        .field_attribute("created_at", "#[diesel(deserialize_as = \"std::time::SystemTime\")]")
        // Disable this for now
        .field_attribute("updated_at", "#[diesel(deserialize_as = \"std::time::SystemTime\")]");

    // .compile_well_known_types();

    tonic_build::configure().compile_with_config(config, &["proto/xpc.proto"], &["proto"])?;
    Ok(())
}
