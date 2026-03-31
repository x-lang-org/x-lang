#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
X 语言测试套件运行器

测试每个流水线步骤的输出是否正确：
- 词法分析 (tokens)
- 语法分析 (ast)
- 类型检查 (typecheck)
- 高层IR (hir)
- 中层IR (mir)
- 低层IR (lir)
- 运行时执行 (runtime)

用法:
    python tests/run_tests.py                    # 运行所有测试
    python tests/run_tests.py --category lexical # 运行特定类别
    python tests/run_tests.py tests/lexical/keywords/basic.toml  # 运行单个测试
"""

import argparse
import io
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Optional

# 设置标准输出编码为 UTF-8
if sys.platform == 'win32':
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

try:
    import tomllib
except ImportError:
    import tomli as tomllib


@dataclass
class TestResult:
    """单个测试的结果"""
    name: str
    path: Path
    passed: bool
    stage_results: dict = field(default_factory=dict)
    error_message: Optional[str] = None
    duration_ms: float = 0.0


@dataclass
class TestConfig:
    """测试配置"""
    name: str
    description: str = ""
    category: str = ""
    spec: list = field(default_factory=list)
    source: str = ""
    expect: dict = field(default_factory=dict)
    compile_fail: bool = False
    error_contains: list = field(default_factory=list)


class TestRunner:
    """测试运行器"""

    def __init__(self, project_root: Path, verbose: bool = False):
        self.project_root = project_root
        self.verbose = verbose
        self.cli_path = project_root / "tools" / "x-cli"
        self.tests_dir = project_root / "tests"

    def discover_tests(self, category: Optional[str] = None) -> list[Path]:
        """发现所有测试文件"""
        tests = []
        test_dir = self.tests_dir

        if category:
            test_dir = test_dir / category

        for toml_file in test_dir.rglob("*.toml"):
            # 跳过配置文件
            if toml_file.name == "config.toml":
                continue
            tests.append(toml_file)

        return sorted(tests)

    def parse_test(self, test_path: Path) -> TestConfig:
        """解析测试文件"""
        with open(test_path, "rb") as f:
            data = tomllib.load(f)

        return TestConfig(
            name=data.get("name", test_path.stem),
            description=data.get("description", ""),
            category=data.get("category", ""),
            spec=data.get("spec", []),
            source=data.get("source", ""),
            expect=data.get("expect", {}),
            compile_fail=data.get("compile_fail", False),
            error_contains=data.get("error_contains", []),
        )

    def run_cli(self, args: list, input_source: Optional[str] = None) -> tuple[int, str, str]:
        """运行 CLI 命令"""
        cmd = ["cargo", "run", "--"] + args

        # 设置环境变量以强制 UTF-8 输出
        env = os.environ.copy()
        env['PYTHONIOENCODING'] = 'utf-8'
        if sys.platform == 'win32':
            env['PYTHONUTF8'] = '1'

        result = subprocess.run(
            cmd,
            cwd=self.cli_path,
            capture_output=True,
            text=True,
            input=input_source,
            timeout=120,
            encoding='utf-8',
            errors='replace',
            env=env,
        )

        return result.returncode, result.stdout, result.stderr

    def get_tokens(self, source: str) -> tuple[bool, str]:
        """获取词法分析结果"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["compile", temp_path, "--emit", "tokens"]
            )
            return returncode == 0, stdout
        finally:
            os.unlink(temp_path)

    def get_ast(self, source: str) -> tuple[bool, str]:
        """获取 AST 结果"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["compile", temp_path, "--emit", "ast"]
            )
            return returncode == 0, stdout
        finally:
            os.unlink(temp_path)

    def get_hir(self, source: str) -> tuple[bool, str]:
        """获取 HIR 结果"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["compile", temp_path, "--emit", "hir"]
            )
            return returncode == 0, stdout
        finally:
            os.unlink(temp_path)

    def get_mir(self, source: str) -> tuple[bool, str]:
        """获取 MIR 结果"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["compile", temp_path, "--emit", "mir"]
            )
            return returncode == 0, stdout
        finally:
            os.unlink(temp_path)

    def get_lir(self, source: str) -> tuple[bool, str]:
        """获取 LIR 结果"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["compile", temp_path, "--emit", "lir"]
            )
            return returncode == 0, stdout
        finally:
            os.unlink(temp_path)

    def run_program(self, source: str) -> tuple[bool, str, int]:
        """运行程序并获取输出"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["run", temp_path]
            )
            return returncode == 0, stdout, returncode
        finally:
            os.unlink(temp_path)

    def check_program(self, source: str) -> tuple[bool, str]:
        """检查程序类型"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.x', delete=False, encoding='utf-8') as f:
            f.write(source)
            temp_path = f.name

        try:
            returncode, stdout, stderr = self.run_cli(
                ["check", temp_path]
            )
            return returncode == 0, stderr
        finally:
            os.unlink(temp_path)

    def verify_tokens(self, output: str, expected: dict) -> tuple[bool, str]:
        """验证词法分析结果"""
        errors = []

        # 检查必须包含的 token 类型
        if "contains" in expected:
            for token_type in expected["contains"]:
                if token_type not in output:
                    errors.append(f"缺少 token 类型: {token_type}")

        # 检查 token 数量
        if "count" in expected:
            # 简单统计非空行数
            lines = [l for l in output.split('\n') if l.strip()]
            if len(lines) != expected["count"]:
                errors.append(f"token 数量不匹配: 期望 {expected['count']}, 实际 {len(lines)}")

        # 检查不应包含的 token
        if "not_contains" in expected:
            for token_type in expected["not_contains"]:
                if token_type in output:
                    errors.append(f"不应包含 token 类型: {token_type}")

        return len(errors) == 0, "\n".join(errors)

    def verify_ast(self, output: str, expected: dict) -> tuple[bool, str]:
        """验证 AST 结果"""
        errors = []

        # 检查是否包含特定声明类型
        if "has_declaration" in expected:
            if expected["has_declaration"]:
                if "Declaration" not in output:
                    errors.append("缺少声明节点")

        if "declaration_type" in expected:
            decl_type = expected["declaration_type"]
            if decl_type not in output:
                errors.append(f"缺少声明类型: {decl_type}")

        # 检查特定节点
        if "nodes" in expected:
            for node in expected["nodes"]:
                if node not in output:
                    errors.append(f"缺少 AST 节点: {node}")

        # 检查不应包含的节点
        if "not_nodes" in expected:
            for node in expected["not_nodes"]:
                if node in output:
                    errors.append(f"不应包含节点: {node}")

        return len(errors) == 0, "\n".join(errors)

    def verify_runtime(self, output: str, returncode: int, expected: dict) -> tuple[bool, str]:
        """验证运行时结果"""
        errors = []

        # 清理输出：移除 CLI 状态信息
        cleaned_output = self._clean_cli_output(output)

        # 检查退出码
        if "exit_code" in expected:
            if returncode != expected["exit_code"]:
                errors.append(f"退出码不匹配: 期望 {expected['exit_code']}, 实际 {returncode}")

        # 检查输出
        if "output" in expected:
            expected_output = expected["output"]
            if cleaned_output != expected_output:
                errors.append(f"输出不匹配:\n期望:\n{repr(expected_output)}\n实际:\n{repr(cleaned_output)}")

        # 检查输出包含
        if "output_contains" in expected:
            for text in expected["output_contains"]:
                if text not in cleaned_output:
                    errors.append(f"输出应包含: {text}")

        return len(errors) == 0, "\n".join(errors)

    def _clean_cli_output(self, output: str) -> str:
        """清理 CLI 输出，移除状态信息"""
        lines = output.split('\n')
        cleaned = []
        for line in lines:
            stripped = line.strip()
            # 移除 CLI 状态行（支持中文）
            if stripped.startswith('Finished'):
                continue
            if stripped.startswith('Running'):
                continue
            if stripped.startswith('Compiling'):
                continue
            # 移除空行（通常是警告后的空行）
            if not stripped:
                continue
            cleaned.append(line)
        # 重新组合并添加末尾换行
        result = '\n'.join(cleaned)
        if result and not result.endswith('\n'):
            result += '\n'
        return result

    def verify_compile_fail(self, stderr: str, expected: dict) -> tuple[bool, str]:
        """验证编译失败"""
        errors = []

        # 检查错误消息包含
        if "error_contains" in expected:
            for text in expected["error_contains"]:
                if text not in stderr:
                    errors.append(f"错误消息应包含: {text}")

        return len(errors) == 0, "\n".join(errors)

    def run_test(self, test_path: Path) -> TestResult:
        """运行单个测试"""
        import time
        start_time = time.time()

        config = self.parse_test(test_path)
        result = TestResult(
            name=config.name,
            path=test_path,
            passed=True,
            stage_results={},
        )

        expect = config.expect

        # 如果是编译失败测试
        if config.compile_fail:
            success, stderr = self.check_program(config.source)
            if success:
                result.passed = False
                result.error_message = "预期编译失败，但编译成功"
            else:
                verify_ok, verify_error = self.verify_compile_fail(stderr, expect)
                result.passed = verify_ok
                result.error_message = verify_error
                result.stage_results["compile_fail"] = verify_ok

            result.duration_ms = (time.time() - start_time) * 1000
            return result

        # 测试词法分析
        if "tokens" in expect:
            success, output = self.get_tokens(config.source)
            if success:
                verify_ok, verify_error = self.verify_tokens(output, expect["tokens"])
                result.stage_results["tokens"] = verify_ok
                if not verify_ok:
                    result.passed = False
                    result.error_message = f"tokens: {verify_error}"
            else:
                result.passed = False
                result.stage_results["tokens"] = False
                result.error_message = f"tokens: 词法分析失败"

        # 测试 AST
        if "ast" in expect:
            success, output = self.get_ast(config.source)
            if success:
                verify_ok, verify_error = self.verify_ast(output, expect["ast"])
                result.stage_results["ast"] = verify_ok
                if not verify_ok:
                    result.passed = False
                    result.error_message = f"ast: {verify_error}"
            else:
                result.passed = False
                result.stage_results["ast"] = False
                result.error_message = f"ast: 语法分析失败"

        # 测试 HIR
        if "hir" in expect:
            success, output = self.get_hir(config.source)
            result.stage_results["hir"] = success
            if not success:
                result.passed = False
                result.error_message = f"hir: HIR 生成失败"

        # 测试 MIR
        if "mir" in expect:
            success, output = self.get_mir(config.source)
            result.stage_results["mir"] = success
            if not success:
                result.passed = False
                result.error_message = f"mir: MIR 生成失败"

        # 测试 LIR
        if "lir" in expect:
            success, output = self.get_lir(config.source)
            result.stage_results["lir"] = success
            if not success:
                result.passed = False
                result.error_message = f"lir: LIR 生成失败"

        # 运行时测试
        if expect.get("compile", True):
            success, output, returncode = self.run_program(config.source)
            if "runtime" in expect:
                verify_ok, verify_error = self.verify_runtime(output, returncode, expect["runtime"])
                result.stage_results["runtime"] = verify_ok
                if not verify_ok:
                    result.passed = False
                    result.error_message = f"runtime: {verify_error}"
            else:
                result.stage_results["runtime"] = success

        result.duration_ms = (time.time() - start_time) * 1000
        return result


def print_result(result: TestResult, verbose: bool = False):
    """打印测试结果"""
    status = "✓ PASS" if result.passed else "✗ FAIL"
    print(f"{status} {result.name} ({result.duration_ms:.1f}ms)")

    if verbose or not result.passed:
        if result.stage_results:
            stages = " | ".join(f"{k}: {'✓' if v else '✗'}" for k, v in result.stage_results.items())
            print(f"    stages: {stages}")
        if result.error_message:
            print(f"    error: {result.error_message}")


def main():
    parser = argparse.ArgumentParser(description="X 语言测试套件运行器")
    parser.add_argument(
        "test_path",
        nargs="?",
        help="测试文件或目录路径",
    )
    parser.add_argument(
        "--category",
        "-c",
        help="运行特定类别的测试 (如: lexical, types, expressions)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="详细输出",
    )
    parser.add_argument(
        "--list",
        "-l",
        action="store_true",
        help="仅列出测试，不运行",
    )

    args = parser.parse_args()

    # 确定项目根目录
    script_dir = Path(__file__).parent
    project_root = script_dir.parent

    runner = TestRunner(project_root, verbose=args.verbose)

    # 发现测试
    if args.test_path:
        test_path = Path(args.test_path)
        # 如果是相对路径，转换为绝对路径
        if not test_path.is_absolute():
            test_path = project_root / test_path
        if test_path.is_file():
            tests = [test_path]
        else:
            tests = runner.discover_tests(str(test_path).replace("tests/", "").replace("tests\\", ""))
    elif args.category:
        tests = runner.discover_tests(args.category)
    else:
        tests = runner.discover_tests()

    if not tests:
        print("未找到测试文件")
        return 1

    # 仅列出测试
    if args.list:
        print(f"发现 {len(tests)} 个测试:")
        for test in tests:
            rel_path = test.relative_to(project_root)
            print(f"  {rel_path}")
        return 0

    # 运行测试
    print(f"运行 {len(tests)} 个测试...\n")

    passed = 0
    failed = 0
    failed_tests = []

    for test_path in tests:
        rel_path = test_path.relative_to(project_root)
        result = runner.run_test(test_path)

        print_result(result, verbose=args.verbose)

        if result.passed:
            passed += 1
        else:
            failed += 1
            failed_tests.append((rel_path, result))

    # 打印摘要
    print(f"\n{'='*50}")
    print(f"测试结果: {passed} 通过, {failed} 失败")

    if failed_tests and args.verbose:
        print(f"\n失败的测试:")
        for path, result in failed_tests:
            print(f"  {path}")
            if result.error_message:
                print(f"    {result.error_message}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
