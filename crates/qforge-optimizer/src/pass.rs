use qforge_ir::Circuit;

#[derive(Debug, Clone)]
pub struct PassReport {
    pub pass_name:     String,
    pub gates_removed: i64,
    pub reason:        String,
}

pub trait Pass: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, circuit: Circuit) -> (Circuit, PassReport);
}
