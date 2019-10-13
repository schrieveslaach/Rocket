#[derive(Debug)]
pub struct Message {

}

unsafe impl Send for Message {}
unsafe impl Sync for Message {}