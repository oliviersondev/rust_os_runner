use limine::request::FramebufferRequest;

#[used]
#[unsafe(link_section = ".limine_requests")]
pub static FRAME_BUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
