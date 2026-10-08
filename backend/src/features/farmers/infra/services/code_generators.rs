use crate::features::farmers::{app::SignInCodeGenerator, domain::SignInCode};

/// A fresh random code for every request.
#[derive(Debug, Default)]
pub struct RandomSignInCodeGenerator;

impl SignInCodeGenerator for RandomSignInCodeGenerator {
    fn generate(&self) -> SignInCode {
        SignInCode::from_number(rand::random_range(0..1_000_000))
    }
}

/// Always the same code. For demos and local work where no message can be
/// delivered; anyone who knows the code can sign in as any phone.
#[derive(Debug)]
pub struct FixedSignInCodeGenerator {
    code: SignInCode,
}

impl FixedSignInCodeGenerator {
    pub fn new(code: SignInCode) -> Self {
        Self { code }
    }
}

impl SignInCodeGenerator for FixedSignInCodeGenerator {
    fn generate(&self) -> SignInCode {
        self.code.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_random_code_is_always_six_digits() {
        for _ in 0..100 {
            let code = RandomSignInCodeGenerator.generate();

            assert!(SignInCode::new(code.as_str().to_string()).is_ok());
        }
    }

    #[test]
    fn the_fixed_generator_repeats_its_code() {
        let generator = FixedSignInCodeGenerator::new(SignInCode::from_number(123_456));

        assert_eq!(generator.generate().as_str(), "123456");
        assert_eq!(generator.generate().as_str(), "123456");
    }
}
