// X语言Web Framework - 模板引擎模块
//
// 提供模板的解析和渲染功能

import "../collections"

/// 模板引擎类型
type TemplateEngine {
  templates: Map<String, String>,
  base_dir: String,
  cache: Map<String, String>,
  enable_cache: Bool,
}

/// 创建模板引擎
///
/// # 参数
/// - `base_dir`: 模板文件基础目录
///
/// # 返回值
/// - `TemplateEngine`: 新创建的模板引擎对象
function template_engine(base_dir: String): TemplateEngine {
  {
    templates: map(),
    base_dir: base_dir,
    cache: map(),
    enable_cache: true,
  }
}

/// 设置模板缓存
///
/// # 参数
/// - `engine`: 模板引擎对象
/// - `enable`: 是否启用缓存
///
/// # 返回值
/// - `TemplateEngine`: 更新后的模板引擎对象
function set_cache(engine: TemplateEngine, enable: Bool): TemplateEngine {
  engine.enable_cache = enable
  if !enable {
    engine.cache = map()
  }
  engine
}

/// 加载模板文件
///
/// # 参数
/// - `engine`: 模板引擎对象
/// - `name`: 模板名称
/// - `path`: 模板文件路径
///
/// # 返回值
/// - `TemplateEngine`: 更新后的模板引擎对象
function load_template(engine: TemplateEngine, name: String, path: String): TemplateEngine {
  // 简化实现，使用内存中的模板内容
  let content = "Template content for " + name
  map_set(engine.templates, name, content)
  // 清除缓存
  if engine.enable_cache {
    map_remove(engine.cache, name)
  }
  engine
}

/// 渲染模板
///
/// # 参数
/// - `engine`: 模板引擎对象
/// - `name`: 模板名称
/// - `context`: 模板上下文（变量）
///
/// # 返回值
/// - `String`: 渲染后的模板内容
function render_template(engine: TemplateEngine, name: String, context: Map<String, Any>): String {
  if !map_contains(engine.templates, name) {
    return "Template not found: " + name
  }
  
  // 检查缓存
  if engine.enable_cache && map_contains(engine.cache, name) {
    let cached = map_get(engine.cache, name)
    return render(cached, context)
  }
  
  let template = map_get(engine.templates, name)
  // 处理模板继承和包含
  let processed_template = process_template(engine, template, context)
  
  // 缓存处理后的模板
  if engine.enable_cache {
    map_set(engine.cache, name, processed_template)
  }
  
  render(processed_template, context)
}

/// 处理模板继承和包含
///
/// # 参数
/// - `engine`: 模板引擎对象
/// - `template`: 模板内容
/// - `context`: 模板上下文
///
/// # 返回值
/// - `String`: 处理后的模板内容
function process_template(engine: TemplateEngine, template: String, context: Map<String, Any>): String {
  let result = template
  
  // 处理模板继承 {% extends "template" %}
  let pattern = "\\{% extends \"([^\"]+)\" %}"
  let matches = regex_find_all(pattern, result)
  
  for match in matches {
    if len(match) >= 2 {
      let parent_template = trim(match[1])
      if map_contains(engine.templates, parent_template) {
        let parent_content = map_get(engine.templates, parent_template)
        // 处理块 {% block name %}
        result = process_blocks(result, parent_content)
      }
      // 移除extends指令
      result = replace(result, match[0], "")
    }
  }
  
  // 处理模板包含 {% include "template" %}
  pattern = "\\{% include \"([^\"]+)\" %}"
  matches = regex_find_all(pattern, result)
  
  for match in matches {
    if len(match) >= 2 {
      let include_template = trim(match[1])
      if map_contains(engine.templates, include_template) {
        let include_content = map_get(engine.templates, include_template)
        let processed_include = process_template(engine, include_content, context)
        result = replace(result, match[0], processed_include)
      }
    }
  }
  
  result
}

/// 处理模板块
///
/// # 参数
/// - `child_template`: 子模板内容
/// - `parent_template`: 父模板内容
///
/// # 返回值
/// - `String`: 处理后的模板内容
function process_blocks(child_template: String, parent_template: String): String {
  let result = parent_template
  
  // 提取子模板中的块
  let pattern = "\\{% block ([^%]+) %\\}([\\s\\S]*?)\\{% endblock %}"
  let matches = regex_find_all(pattern, child_template)
  
  for match in matches {
    if len(match) >= 3 {
      let block_name = trim(match[1])
      let block_content = match[2]
      // 替换父模板中的对应块
      let block_pattern = "\\{% block " + block_name + " %\\}([\\s\\S]*?)\\{% endblock %}"
      let block_matches = regex_find_all(block_pattern, result)
      if len(block_matches) > 0 {
        result = replace(result, block_matches[0][0], "{% block " + block_name + " %}" + block_content + "{% endblock %}")
      }
    }
  }
  
  result
}

/// 渲染模板字符串
///
/// # 参数
/// - `template`: 模板字符串
/// - `context`: 模板上下文（变量）
///
/// # 返回值
/// - `String`: 渲染后的内容
function render(template: String, context: Map<String, Any>): String {
  let result = template
  
  // 替换变量 {{variable}}
  let pattern = "\\{\\{([^\\}]+)\\}\\}"
  let matches = regex_find_all(pattern, template)
  
  for match in matches {
    if len(match) >= 2 {
      let var_name = trim(match[1])
      if map_contains(context, var_name) {
        let value = map_get(context, var_name)
        let value_str = to_string(value)
        result = replace(result, "{{" + var_name + "}}", value_str)
      }
    }
  }
  
  // 处理条件语句 {% if condition %}
  pattern = "\\{% if ([^%]+) %\\}([\\s\\S]*?)\\{% endif %}"
  matches = regex_find_all(pattern, result)
  
  for match in matches {
    if len(match) >= 3 {
      let condition = trim(match[1])
      let content = match[2]
      let should_render = false
      
      // 简单的条件处理，只支持变量存在性检查
      if map_contains(context, condition) {
        let value = map_get(context, condition)
        should_render = value != false && value != null && value != ""
      }
      
      if should_render {
        result = replace(result, match[0], content)
      } else {
        result = replace(result, match[0], "")
      }
    }
  }
  
  // 处理循环语句 {% for item in items %}
  pattern = "\\{% for ([^ ]+) in ([^%]+) %\\}([\\s\\S]*?)\\{% endfor %}"
  matches = regex_find_all(pattern, result)
  
  for match in matches {
    if len(match) >= 4 {
      let item_var = match[1]
      let items_var = trim(match[2])
      let content = match[3]
      let rendered = ""
      
      if map_contains(context, items_var) {
        let items = map_get(context, items_var)
        if is_list(items) {
          for item in items {
            let item_context = map_copy(context)
            map_set(item_context, item_var, item)
            rendered = rendered + render(content, item_context)
          }
        }
      }
      
      result = replace(result, match[0], rendered)
    }
  }
  
  result
}

/// 从字符串创建模板
///
/// # 参数
/// - `engine`: 模板引擎对象
/// - `name`: 模板名称
/// - `content`: 模板内容
///
/// # 返回值
/// - `TemplateEngine`: 更新后的模板引擎对象
function template_from_string(engine: TemplateEngine, name: String, content: String): TemplateEngine {
  map_set(engine.templates, name, content)
  // 清除缓存
  if engine.enable_cache {
    map_remove(engine.cache, name)
  }
  engine
}

/// 清除模板缓存
///
/// # 参数
/// - `engine`: 模板引擎对象
///
/// # 返回值
/// - `TemplateEngine`: 更新后的模板引擎对象
function clear_cache(engine: TemplateEngine): TemplateEngine {
  engine.cache = map()
  engine
}
