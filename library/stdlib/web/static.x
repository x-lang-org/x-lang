// X语言Web Framework - 静态文件服务模块
//
// 提供静态文件的读取和服务功能

import "../io"
import "../sys"

/// 静态文件服务配置类型
type StaticConfig {
  root: String,
  index: String,
  enable_dir_listing: Bool,
}

/// 创建静态文件服务配置
///
/// # 参数
/// - `root`: 静态文件根目录
/// - `index`: 索引文件名
/// - `enable_dir_listing`: 是否启用目录浏览
///
/// # 返回值
/// - `StaticConfig`: 静态文件服务配置
function static_config(root: String, index: String, enable_dir_listing: Bool): StaticConfig {
  {
    root: root,
    index: index,
    enable_dir_listing: enable_dir_listing,
  }
}

/// 创建静态文件服务处理器
///
/// # 参数
/// - `config`: 静态文件服务配置
///
/// # 返回值
/// - `(Request) -> Response`: 静态文件服务处理器
function static_handler(config: StaticConfig): (Request) -> Response {
  fn(request: Request): Response {
    let path = request.path
    
    // 模拟静态文件服务
    let content = "Static file content for " + path
    let content_type = get_content_type(path)
    let headers = map()
    map_set(headers, "Content-Type", content_type)
    
    build_response(200, content, headers)
  }
}

/// 获取文件的Content-Type
///
/// # 参数
/// - `file_path`: 文件路径
///
/// # 返回值
/// - `String`: Content-Type
function get_content_type(file_path: String): String {
  let extension = path_extension(file_path)
  
  match extension {
    "html" => "text/html; charset=utf-8",
    "htm" => "text/html; charset=utf-8",
    "css" => "text/css; charset=utf-8",
    "js" => "application/javascript; charset=utf-8",
    "json" => "application/json; charset=utf-8",
    "xml" => "application/xml; charset=utf-8",
    "txt" => "text/plain; charset=utf-8",
    "md" => "text/markdown; charset=utf-8",
    "jpg" => "image/jpeg",
    "jpeg" => "image/jpeg",
    "png" => "image/png",
    "gif" => "image/gif",
    "webp" => "image/webp",
    "svg" => "image/svg+xml",
    "ico" => "image/x-icon",
    "pdf" => "application/pdf",
    "zip" => "application/zip",
    "rar" => "application/x-rar-compressed",
    "tar" => "application/x-tar",
    "gz" => "application/gzip",
    "mp3" => "audio/mpeg",
    "mp4" => "video/mp4",
    "webm" => "video/webm",
    "ogg" => "audio/ogg",
    _ => "application/octet-stream"
  }
}

/// 生成目录列表
///
/// # 参数
/// - `dir_path`: 目录路径
/// - `url_path`: URL路径
///
/// # 返回值
/// - `String`: 目录列表HTML
function generate_dir_listing(dir_path: String, url_path: String): String {
  let files = list_dir(dir_path)
  let mut html = "<!DOCTYPE html><html><head><title>Directory Listing</title></head><body><h1>Directory Listing: " + url_path + "</h1><ul>"
  
  // 添加上级目录链接
  if url_path != "/" {
    let parent_path = path_dirname(url_path)
    if parent_path == "" {
      parent_path = "/"
    }
    html = html + "<li><a href=\"" + parent_path + "\">..</a></li>"
  }
  
  // 添加文件和子目录链接
  for file in files {
    let full_path = dir_path + "/" + file
    let file_url = url_path
    if file_url != "/" {
      file_url = file_url + "/"
    }
    file_url = file_url + file
    
    if is_dir(full_path) {
      html = html + "<li><a href=\"" + file_url + "/\">" + file + "/</a></li>"
    } else {
      html = html + "<li><a href=\"" + file_url + "">" + file + "</a></li>"
    }
  }
  
  html = html + "</ul></body></html>"
  html
}
