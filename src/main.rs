struct TaskType1 {
    id: u32,
    words : [u32;1]
}

struct TaskType2 {
    id: u32,
    words : [u32;2]
}

// Common Traits
trait BorrowId {
    type Output;
    fn borrow_id(&self) -> &Self::Output;
}

trait BorrowData {
    fn borrow_data(&self) -> &[u32];
}

// Manual Implementation of traits
impl BorrowData for TaskType1 {
    fn borrow_data(&self) -> &[u32] {
        &self.words
    }
}

impl BorrowData for TaskType2 {
    fn borrow_data(&self) -> &[u32] {
        &self.words
    }
}

// implement traits with macros
macro_rules! impl_BorrowId {
    ($out:ty, [$($t:ty),+]) => {
        $(impl BorrowId for $t {
            type Output = $out;
            fn borrow_id(&self) -> &Self::Output {
                &self.id
            }
        })*
    }
}

impl_BorrowId!{u32, [TaskType1, TaskType2]}

// A function that operators any T with traits BorrowID and BorrowData
fn process_common<T: BorrowId<Output=u32>+BorrowData>(arg: T) {
    println!("type: {}", arg.borrow_id());
    for w in arg.borrow_data() {
        println!("words: {}", w);
    }
}

fn main() {
    let t1 = TaskType1 { id: 0, words: [1] };
    let t2 = TaskType2 { id: 2, words: [2,3] };

    process_common(t1);
    process_common(t2);
}
