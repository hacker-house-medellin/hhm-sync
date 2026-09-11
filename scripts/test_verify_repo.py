#!/usr/bin/env python3
import unittest

from verify_repo import credential_shaped


class CredentialShapeTests(unittest.TestCase):
    def test_detects_long_github_and_linear_tokens(self) -> None:
        self.assertTrue(credential_shaped("ghp_" + "A" * 24))
        self.assertTrue(credential_shaped("lin_api_" + "B" * 24))

    def test_detects_plausible_private_key_material(self) -> None:
        block = "-----BEGIN PRIVATE KEY-----\n" + "A" * 64 + "\n-----END PRIVATE KEY-----"
        self.assertTrue(credential_shaped(block))

    def test_allows_header_only_documentation_examples(self) -> None:
        example = 'JWT_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\\nMIIE…\\n-----END PRIVATE KEY-----\\n"'
        self.assertFalse(credential_shaped(example))
        self.assertFalse(credential_shaped("-----BEGIN PRIVATE KEY-----"))

    def test_allows_short_obviously_synthetic_token_labels(self) -> None:
        self.assertFalse(credential_shaped("ghp_example"))
        self.assertFalse(credential_shaped("lin_api_placeholder"))


if __name__ == "__main__":
    unittest.main()
