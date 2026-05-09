#![allow(dead_code)]

mod commands;
mod config;
mod lockfile;
mod manifest;
mod pipeline;
mod project;
mod registry;
mod resolver;
mod utils;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "x")]
#[command(version = "0.1.0")]
#[command(about = "X语言工具链 — 构建、运行、测试、发布X语言项目")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 使用详细输出
    #[arg(short, long, global = true)]
    verbose: bool,

    /// 不输出日志信息
    #[arg(short, long, global = true)]
    quiet: bool,

    /// 控制颜色输出: auto, always, never
    #[arg(long, global = true, default_value = "auto")]
    color: String,

    /// 在指定目录中运行
    #[arg(short = 'C', long, global = true)]
    directory: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    // ── Build commands ──────────────────────────────────────────────
    /// 构建当前项目
    Build {
        /// 使用 release 配置构建
        #[arg(long)]
        release: bool,
        /// 构建目标平台
        #[arg(long)]
        target: Option<String>,
        /// 并行作业数
        #[arg(short, long)]
        jobs: Option<u32>,
        /// 启用指定特性
        #[arg(short = 'F', long)]
        features: Vec<String>,
        /// 启用所有特性
        #[arg(long)]
        all_features: bool,
        /// 不使用默认特性
        #[arg(long)]
        no_default_features: bool,
        /// 构建所有示例
        #[arg(long)]
        examples: bool,
    },

    /// 运行X语言程序
    Run {
        /// 源文件路径（单文件模式）
        #[arg(value_name = "FILE")]
        file: Option<String>,
        /// 使用 release 配置
        #[arg(long)]
        release: bool,
        /// 运行指定示例
        #[arg(long)]
        example: Option<String>,
        /// 运行指定二进制目标
        #[arg(long)]
        bin: Option<String>,
        /// 仅输出程序 print 结果
        #[arg(short, long)]
        quiet: bool,
        /// 传递给程序的参数
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// 检查项目语法和类型（不生成代码）
    Check {
        /// 源文件路径（单文件模式）
        #[arg(value_name = "FILE")]
        file: Option<String>,
        /// 检查所有目标（含测试、示例等）
        #[arg(long)]
        all_targets: bool,
    },

    /// 编译X语言源代码到目标文件/可执行文件
    Compile {
        #[arg(value_name = "FILE")]
        file: String,
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
        /// 输出中间结果: tokens, ast, hir, mir, lir, zig, dotnet, csharp, rust
        #[arg(long, value_name = "STAGE")]
        emit: Option<String>,
        /// 仅生成目标文件，不链接
        #[arg(long)]
        no_link: bool,
        /// 使用 release 优化构建
        #[arg(long)]
        release: bool,
        /// 构建目标平台 (native, wasm32-wasi)
        #[arg(long)]
        target: Option<String>,
    },

    /// 运行项目测试
    Test {
        /// 过滤测试名称
        #[arg(value_name = "FILTER")]
        filter: Option<String>,
        /// integration 测试分类
        #[arg(value_name = "CATEGORY")]
        category: Option<String>,
        /// 使用 release 配置
        #[arg(long)]
        release: bool,
        /// 仅测试库
        #[arg(long)]
        lib: bool,
        /// 运行文档测试
        #[arg(long)]
        doc: bool,
        /// 仅编译不运行
        #[arg(long)]
        no_run: bool,
        /// 并行作业数
        #[arg(short, long)]
        jobs: Option<u32>,
    },

    /// 运行项目基准测试
    Bench {
        /// 过滤基准名称
        #[arg(value_name = "FILTER")]
        filter: Option<String>,
        /// 仅编译不运行
        #[arg(long)]
        no_run: bool,
    },

    /// 清除构建产物
    Clean {
        /// 仅清除文档
        #[arg(long)]
        doc: bool,
        /// 仅清除 release 目录
        #[arg(long)]
        release: bool,
    },

    /// 生成项目文档
    Doc {
        /// 在浏览器中打开
        #[arg(long)]
        open: bool,
        /// 不生成依赖的文档
        #[arg(long)]
        no_deps: bool,
        /// 包含私有项的文档
        #[arg(long)]
        document_private_items: bool,
    },

    /// 获取依赖（不构建）
    Fetch,

    /// 自动修复代码中的警告
    Fix {
        /// 允许在有未提交更改时修复
        #[arg(long)]
        allow_dirty: bool,
        /// 允许在有暂存更改时修复
        #[arg(long)]
        allow_staged: bool,
    },

    // ── Manifest / dependency commands ──────────────────────────────
    /// 添加依赖到 x.toml
    Add {
        /// 要添加的包名 (格式: name 或 name@version)
        #[arg(required = true)]
        packages: Vec<String>,
        /// 添加为开发依赖
        #[arg(long)]
        dev: bool,
        /// 添加为构建依赖
        #[arg(long)]
        build: bool,
        /// 标记为可选依赖
        #[arg(long)]
        optional: bool,
        /// 重命名依赖
        #[arg(long)]
        rename: Option<String>,
        /// 使用本地路径
        #[arg(long)]
        path: Option<String>,
        /// 从 Git 仓库添加
        #[arg(long)]
        git: Option<String>,
        /// Git 分支
        #[arg(long)]
        branch: Option<String>,
        /// Git 标签
        #[arg(long)]
        tag: Option<String>,
        /// Git 修订版本
        #[arg(long)]
        rev: Option<String>,
        /// 启用指定特性
        #[arg(short = 'F', long)]
        features: Vec<String>,
        /// 禁用默认特性
        #[arg(long)]
        no_default_features: bool,
    },

    /// 从 x.toml 移除依赖
    Remove {
        /// 要移除的包名
        #[arg(required = true)]
        packages: Vec<String>,
        /// 从开发依赖移除
        #[arg(long)]
        dev: bool,
        /// 从构建依赖移除
        #[arg(long)]
        build: bool,
    },

    /// 生成或更新 x.lock
    #[command(name = "generate-lockfile")]
    GenerateLockfile,

    /// 输出项目清单路径 (JSON)
    #[command(name = "locate-project")]
    LocateProject {
        /// 查找工作区根
        #[arg(long)]
        workspace: bool,
    },

    /// 以 JSON 格式输出项目元数据
    Metadata {
        /// 不包含依赖解析
        #[arg(long)]
        no_deps: bool,
        /// 格式版本
        #[arg(long)]
        format_version: Option<u32>,
    },

    /// 输出完整的包标识符
    Pkgid {
        /// 指定包名
        #[arg(value_name = "SPEC")]
        spec: Option<String>,
    },

    /// 显示依赖树
    Tree {
        /// 最大深度
        #[arg(short, long)]
        depth: Option<usize>,
        /// 反转依赖方向
        #[arg(short, long)]
        invert: bool,
        /// 不合并重复项
        #[arg(long)]
        no_dedupe: bool,
        /// 前缀样式: indent, depth, none
        #[arg(long, default_value = "indent")]
        prefix: String,
    },

    /// 更新 x.lock 中的依赖版本
    Update {
        /// 仅更新指定的包
        #[arg(value_name = "PACKAGES")]
        packages: Vec<String>,
        /// 激进更新（忽略锁文件约束）
        #[arg(long)]
        aggressive: bool,
        /// 试运行，不实际修改
        #[arg(long)]
        dry_run: bool,
    },

    /// 将依赖复制到本地目录
    Vendor {
        /// 输出目录
        #[arg(value_name = "PATH")]
        path: Option<String>,
        /// 不删除现有 vendor 目录
        #[arg(long)]
        no_delete: bool,
        /// 使用带版本号的目录名
        #[arg(long)]
        versioned_dirs: bool,
    },

    // ── Package commands ───────────────────────────────────────────
    /// 在当前目录初始化新项目
    Init {
        /// 项目路径
        #[arg(value_name = "PATH")]
        path: Option<String>,
        /// 创建库项目
        #[arg(long)]
        lib: bool,
        /// 版本控制系统: git, none
        #[arg(long, default_value = "git")]
        vcs: String,
        /// X语言版本
        #[arg(long)]
        edition: Option<String>,
    },

    /// 创建新项目
    New {
        /// 项目名称/路径
        #[arg(required = true)]
        name: String,
        /// 创建库项目
        #[arg(long)]
        lib: bool,
        /// 版本控制系统: git, none
        #[arg(long, default_value = "git")]
        vcs: String,
        /// X语言版本
        #[arg(long)]
        edition: Option<String>,
    },

    /// 安装X语言可执行包
    #[command(disable_version_flag = true)]
    Install {
        /// 要安装的包名
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
        /// 从本地路径安装
        #[arg(long)]
        path: Option<String>,
        /// 从 Git 仓库安装
        #[arg(long)]
        git: Option<String>,
        /// 指定版本
        #[arg(long)]
        version: Option<String>,
        /// 强制覆盖安装
        #[arg(short, long)]
        force: bool,
        /// 安装目录
        #[arg(long)]
        root: Option<String>,
        /// 列出已安装的包
        #[arg(long)]
        list: bool,
    },

    /// 卸载已安装的可执行包
    Uninstall {
        /// 要卸载的包名
        #[arg(required = true)]
        packages: Vec<String>,
        /// 安装目录
        #[arg(long)]
        root: Option<String>,
    },

    /// 在注册表中搜索包
    Search {
        /// 搜索关键词
        #[arg(required = true)]
        query: String,
        /// 最大结果数
        #[arg(long, default_value = "10")]
        limit: usize,
        /// 指定注册表
        #[arg(long)]
        registry: Option<String>,
    },

    // ── Publishing commands ────────────────────────────────────────
    /// 登录到包注册表
    Login {
        /// API token
        token: Option<String>,
        /// 指定注册表
        #[arg(long)]
        registry: Option<String>,
    },

    /// 注销包注册表
    Logout {
        /// 指定注册表
        #[arg(long)]
        registry: Option<String>,
    },

    /// 管理包的所有者
    Owner {
        /// 包名
        #[arg(required = true)]
        package: String,
        /// 添加所有者
        #[arg(short, long)]
        add: Vec<String>,
        /// 移除所有者
        #[arg(short, long)]
        remove: Vec<String>,
        /// 列出所有者
        #[arg(short, long)]
        list: bool,
        /// 指定注册表
        #[arg(long)]
        registry: Option<String>,
    },

    /// 将项目打包为可分发的压缩包
    Package {
        /// 列出将要打包的文件
        #[arg(short, long)]
        list: bool,
        /// 跳过验证
        #[arg(long)]
        no_verify: bool,
        /// 允许打包有未提交更改的项目
        #[arg(long)]
        allow_dirty: bool,
        /// 输出目录
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 发布包到注册表
    Publish {
        /// 试运行，不实际发布
        #[arg(long)]
        dry_run: bool,
        /// 跳过验证
        #[arg(long)]
        no_verify: bool,
        /// 允许发布有未提交更改的项目
        #[arg(long)]
        allow_dirty: bool,
        /// 指定注册表
        #[arg(long)]
        registry: Option<String>,
        /// API token
        #[arg(long)]
        token: Option<String>,
    },

    /// 撤回已发布的版本
    #[command(disable_version_flag = true)]
    Yank {
        /// 包名
        #[arg(required = true)]
        package: String,
        /// 版本号
        #[arg(long, required = true)]
        version: String,
        /// 取消撤回
        #[arg(long)]
        undo: bool,
        /// 指定注册表
        #[arg(long)]
        registry: Option<String>,
    },

    // ── Tool commands ──────────────────────────────────────────────
    /// 格式化X语言源代码
    Fmt {
        /// 源文件路径
        #[arg(value_name = "FILE")]
        file: Option<String>,
        /// 仅检查格式，不修改
        #[arg(long)]
        check: bool,
        /// 格式化所有文件（含测试、示例等）
        #[arg(long)]
        all: bool,
    },

    /// 代码检查（类似 clippy）
    Lint {
        /// 自动修复
        #[arg(long)]
        fix: bool,
        /// 允许的 lint
        #[arg(short = 'A', long)]
        allow: Vec<String>,
        /// 拒绝的 lint
        #[arg(short = 'D', long)]
        deny: Vec<String>,
        /// 警告的 lint
        #[arg(short = 'W', long)]
        warn: Vec<String>,
    },

    /// 启动X语言 REPL
    Repl {
        /// REPL 目标平台 (interpreter, js)
        #[arg(long, default_value = "interpreter")]
        target: String,
    },

    /// 管理全局配置
    Config {
        /// 操作: get, set, list
        #[arg(required = true)]
        action: String,
        /// 配置键
        key: Option<String>,
        /// 配置值
        value: Option<String>,
    },

    /// 显示版本信息
    Version,
}

fn main() {
    // 某些路径（尤其是解析/类型检查）在 Windows 上可能触发较深递归，
    // 这里用更大的栈空间运行整个 CLI 以避免 stack overflow。
    let handle = std::thread::Builder::new()
        .name("x-cli-main".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(real_main)
        .unwrap_or_else(|e| {
            utils::error(&format!("无法启动主线程: {}", e));
            std::process::exit(1);
        });

    match handle.join() {
        Ok(()) => {}
        Err(_) => {
            utils::error("主线程崩溃");
            std::process::exit(1);
        }
    }
}

fn real_main() {
    let cli = Cli::parse();

    utils::init_logging(cli.verbose, cli.quiet);

    if let Some(ref dir) = cli.directory {
        if let Err(e) = std::env::set_current_dir(dir) {
            utils::error(&format!("无法切换到目录 {}: {}", dir, e));
            std::process::exit(1);
        }
    }

    let result = dispatch(cli);

    if let Err(e) = result {
        utils::error(&e);
        std::process::exit(1);
    }
}

fn dispatch(cli: Cli) -> Result<(), String> {
    match cli.command {
        // Build commands
        Commands::Build {
            release,
            target,
            jobs,
            features,
            all_features,
            no_default_features,
            examples,
        } => commands::build::exec(
            release,
            target.as_deref(),
            jobs,
            &features,
            all_features,
            no_default_features,
            cli.verbose,
            examples,
        ),

        Commands::Run {
            file,
            release,
            example,
            bin,
            quiet,
            args,
        } => commands::run::exec(
            file.as_deref(),
            release,
            example.as_deref(),
            bin.as_deref(),
            &args,
            quiet || cli.quiet,
        )
        .map(|_| ()),

        Commands::Check { file, all_targets } => {
            commands::check::exec(file.as_deref(), all_targets)
        }

        Commands::Compile {
            file,
            output,
            emit,
            no_link,
            release,
            target,
        } => commands::compile::exec(
            &file,
            output.as_deref(),
            emit.as_deref(),
            no_link,
            release,
            target.as_deref(),
        ),

        Commands::Test {
            filter,
            category,
            release,
            lib,
            doc,
            no_run,
            jobs,
        } => {
            let combined_filter = match (filter.as_deref(), category.as_deref()) {
                (Some("integration"), Some(category)) => Some(format!("integration/{category}")),
                (Some(filter), None) => Some(filter.to_string()),
                (Some(_), Some(category)) => {
                    return Err(format!(
                        "额外的测试分类 `{}` 仅支持与 `x test integration <category>` 一起使用",
                        category
                    ));
                }
                (None, Some(category)) => Some(category.to_string()),
                (None, None) => None,
            };
            commands::test_cmd::exec(
                combined_filter.as_deref(),
                release,
                lib,
                doc,
                no_run,
                jobs,
                cli.verbose,
            )
        }

        Commands::Bench { filter, no_run } => commands::bench::exec(filter.as_deref(), no_run),

        Commands::Clean { doc, release } => commands::clean::exec(doc, release),

        Commands::Doc {
            open,
            no_deps,
            document_private_items,
        } => commands::doc::exec(open, no_deps, document_private_items),

        Commands::Fetch => commands::fetch::exec(),

        Commands::Fix {
            allow_dirty,
            allow_staged,
        } => commands::fix::exec(allow_dirty, allow_staged),

        // Manifest / dependency commands
        Commands::Add {
            packages,
            dev,
            build,
            optional,
            rename,
            path,
            git,
            branch,
            tag,
            rev,
            features,
            no_default_features,
        } => commands::add::exec(
            &packages,
            dev,
            build,
            optional,
            rename.as_deref(),
            path.as_deref(),
            git.as_deref(),
            branch.as_deref(),
            tag.as_deref(),
            rev.as_deref(),
            &features,
            no_default_features,
        ),

        Commands::Remove {
            packages,
            dev,
            build,
        } => commands::remove::exec(&packages, dev, build),

        Commands::GenerateLockfile => commands::generate_lockfile::exec(),

        Commands::LocateProject { workspace } => commands::locate_project::exec(workspace),

        Commands::Metadata {
            no_deps,
            format_version,
        } => commands::metadata::exec(no_deps, format_version),

        Commands::Pkgid { spec } => commands::pkgid::exec(spec.as_deref()),

        Commands::Tree {
            depth,
            invert,
            no_dedupe,
            prefix,
        } => commands::tree::exec(depth, invert, no_dedupe, &prefix),

        Commands::Update {
            packages,
            aggressive,
            dry_run,
        } => commands::update::exec(&packages, aggressive, dry_run),

        Commands::Vendor {
            path,
            no_delete,
            versioned_dirs,
        } => commands::vendor::exec(path.as_deref(), no_delete, versioned_dirs),

        // Package commands
        Commands::Init {
            path,
            lib,
            vcs,
            edition,
        } => commands::init::exec(path.as_deref(), lib, &vcs, edition.as_deref()),

        Commands::New {
            name,
            lib,
            vcs,
            edition,
        } => commands::new::exec(&name, lib, &vcs, edition.as_deref()),

        Commands::Install {
            package,
            path,
            git,
            version,
            force,
            root,
            list,
        } => commands::install::exec(
            package.as_deref(),
            path.as_deref(),
            git.as_deref(),
            version.as_deref(),
            force,
            root.as_deref(),
            list,
        ),

        Commands::Uninstall { packages, root } => {
            commands::uninstall::exec(&packages, root.as_deref())
        }

        Commands::Search {
            query,
            limit,
            registry,
        } => commands::search::exec(&query, limit, registry.as_deref()),

        // Publishing commands
        Commands::Login { token, registry } => {
            commands::login::exec(token.as_deref(), registry.as_deref())
        }

        Commands::Logout { registry } => commands::logout::exec(registry.as_deref()),

        Commands::Owner {
            package,
            add,
            remove,
            list,
            registry,
        } => commands::owner::exec(&package, &add, &remove, list, registry.as_deref()),

        Commands::Package {
            list,
            no_verify,
            allow_dirty,
            output,
        } => commands::package::exec(list, no_verify, allow_dirty, output.as_deref()),

        Commands::Publish {
            dry_run,
            no_verify,
            allow_dirty,
            registry,
            token,
        } => commands::publish::exec(
            dry_run,
            no_verify,
            allow_dirty,
            registry.as_deref(),
            token.as_deref(),
        ),

        Commands::Yank {
            package,
            version,
            undo,
            registry,
        } => commands::yank::exec(&package, &version, undo, registry.as_deref()),

        // Tool commands
        Commands::Fmt { file, check, all } => commands::fmt::exec(file.as_deref(), check, all),

        Commands::Lint {
            fix,
            allow,
            deny,
            warn,
        } => commands::lint::exec(fix, allow, deny, warn),

        Commands::Repl { target } => commands::repl::exec(&target),

        Commands::Config { action, key, value } => {
            commands::config_cmd::exec(&action, key.as_deref(), value.as_deref())
        }

        Commands::Version => {
            println!("x {}", env!("CARGO_PKG_VERSION"));
            println!("X语言工具链 v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
