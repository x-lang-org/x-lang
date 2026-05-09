import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("run_tests.py")
SPEC = importlib.util.spec_from_file_location("run_tests_module", MODULE_PATH)
RUN_TESTS = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(RUN_TESTS)


class RunTestsContractTests(unittest.TestCase):
    def test_build_runtime_expectation_promotes_top_level_exit_code(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            runner = RUN_TESTS.TestRunner(Path(temp_dir))

            runtime_expect = runner.build_runtime_expectation(
                {
                    "compile": True,
                    "exit_code": 0,
                    "runtime": {"output": "ok\n"},
                }
            )

            self.assertEqual(runtime_expect, {"output": "ok\n", "exit_code": 0})

    def test_run_test_reports_runtime_failure_stderr_and_exit_code(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            runner = RUN_TESTS.TestRunner(temp_path)
            test_path = temp_path / "runtime_failure.toml"
            test_path.write_text(
                '\n'.join([
                    'name = "runtime_failure"',
                    'source = "println(42)"',
                    '',
                    '[expect]',
                    'compile = true',
                    'exit_code = 0',
                    '',
                    '[expect.runtime]',
                    'output = "42\\n"',
                ]),
                encoding='utf-8',
            )

            runner.run_program = lambda source: (False, "", "error: boom", 1)

            result = runner.run_test(test_path)

            self.assertFalse(result.passed)
            self.assertFalse(result.stage_results["runtime"])
            self.assertIn("程序执行失败（exit code 1）", result.error_message)
            self.assertIn("error: boom", result.error_message)

    def test_run_program_surfaces_real_cli_stderr_on_typecheck_failure(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            runner = RUN_TESTS.TestRunner(MODULE_PATH.parent.parent)
            source_path = temp_path / "type_mismatch.x"
            source_path.write_text("let x: integer = 3.14\n", encoding="utf-8")

            success, stdout, stderr, returncode = runner.run_program(source_path.read_text(encoding="utf-8"))

            self.assertFalse(success)
            self.assertEqual(stdout, "")
            self.assertNotEqual(returncode, 0)
            self.assertIn("类型检查错误", stderr)
            self.assertIn("类型不匹配", stderr)


if __name__ == "__main__":
    unittest.main()
