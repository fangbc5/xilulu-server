// 服务模块

pub mod image_captcha;
pub mod nickname_generator;
pub mod temp_token;
pub mod validation;
pub mod verify_code;

pub use image_captcha::ImageCaptchaService;
pub use nickname_generator::{NicknameConfig, NicknameGenerator, NicknameMode};
pub use temp_token::TempTokenService;
pub use validation::{validate_login_request, validate_register_request};
pub use verify_code::{VerifyCodeService, VerifyCodeType};
