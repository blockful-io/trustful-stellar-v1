#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]

// Structs and variables
pub struct Test {
    pub count: u32,
    pub last_incr: u32,
}

// Contract struct
#[contract]
pub struct ScorerContract;

// Contract implementation
#[contractimpl]
impl ScorerContract {
    pub fn increment(env: Env, incr: u32) -> u32 {
      let test = Test {
        count: 0,
        last_incr: 0,
      };    
      return test.count;
    }
}