#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope<T> {
    pub operation_id: String,
    pub revision: u64,
    pub payload: T,
}
impl<T> Envelope<T> {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.operation_id.trim().is_empty() {
            Err("operation_id is required")
        } else if self.operation_id.len() > 256 {
            Err("operation_id exceeds 256 bytes")
        } else if self.operation_id.chars().any(char::is_control) {
            Err("operation_id contains control characters")
        } else if self.operation_id.trim() != self.operation_id {
            Err("operation_id must not have surrounding whitespace")
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversized_and_control_character_ids() {
        for invalid in ["x".repeat(257), "has\nnewline".to_string(), "has\tTab".to_string()] {
            assert!(Envelope { operation_id: invalid, revision: 1, payload: () }.validate().is_err());
        }
    }

    #[test]
    fn accepts_bounded_unicode_id() {
        assert!(Envelope { operation_id: "reserva-ñ".into(), revision: 0, payload: () }.validate().is_ok());
    }

    #[test]
    fn checks_utf8_byte_boundaries_without_rewriting_operation_identity() {
        let exactly_256 = "ñ".repeat(128);
        assert_eq!(exactly_256.as_bytes().len(), 256);
        assert!(Envelope { operation_id: exactly_256, revision: 0, payload: () }.validate().is_ok());

        let over_limit = format!("{}x", "ñ".repeat(128));
        assert_eq!(over_limit.as_bytes().len(), 257);
        assert_eq!(
            Envelope { operation_id: over_limit, revision: 0, payload: () }.validate(),
            Err("operation_id exceeds 256 bytes")
        );
    }

    #[test]
    fn rejects_ambiguous_leading_or_trailing_whitespace() {
        for id in [" reservation", "reservation ", "\u{2003}reservation"] {
            assert_eq!(
                Envelope { operation_id: id.to_string(), revision: 1, payload: () }.validate(),
                Err("operation_id must not have surrounding whitespace")
            );
        }
        assert!(Envelope { operation_id: "tenant/workspace event".to_owned(), revision: 0, payload: () }.validate().is_ok());
    }

    #[test]
    fn rejects_empty_id() {
        assert!(
            Envelope {
                operation_id: "".into(),
                revision: 1,
                payload: ()
            }
            .validate()
            .is_err()
        );
    }
}
