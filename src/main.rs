use std::fs;
use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use tera::{Context, Tera};
use toml;

#[derive(Serialize)]
struct Post {
    name: String,
    title: String,
    create_date: String,
    tags: Vec<String>,
    content: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析并排序文章（按日期降序）
    let mut posts = parse_posts()?;
    posts.sort_by(|a, b| b.create_date.cmp(&a.create_date));

    // 准备输出目录
    prepare_output_directories()?;

    // 初始化模板引擎
    let tera = init_template_engine()?;

    // 生成首页
    generate_index_page(&tera, &posts)?;

    // 生成文章页面
    generate_post_pages(&tera, &posts)?;

    // 复制静态资源
    copy_static_files()?;

    println!("✅ 站点生成完成，文件保存在dist目录");

    Ok(())
}

/// 解析所有博客文章
fn parse_posts() -> Result<Vec<Post>, io::Error> {
    let mut posts = Vec::new();
    let posts_dir = PathBuf::from("site/posts");

    // 检查posts目录是否存在
    if !posts_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "posts目录不存在，请创建 site/posts 目录",
        ));
    }

    for entry in fs::read_dir(posts_dir)? {
        let entry = entry?;
        let path = entry.path();

        // 跳过非文件项
        if !path.is_file() {
            continue;
        }

        // 获取文件名（不带扩展名）
        let file_name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 读取并解析文件内容
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(post) = parse_post_content(&file_name, &content) {
                posts.push(post);
            }
        }
    }

    Ok(posts)
}

/// 解析单个文章内容
fn parse_post_content(file_name: &str, content: &str) -> Option<Post> {
    // 分割前置元数据和内容
    let (frontmatter, content) = split_frontmatter_and_content(content);

    // 解析前置元数据
    let frontmatter: toml::Value = match toml::from_str(frontmatter) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("无法解析文章 {} 的元数据: {}", file_name, err);
            return None;
        }
    };

    // 提取文章信息
    let title = frontmatter["title"]
        .as_str()
        .unwrap_or("无标题")
        .to_string();

    let date = frontmatter["date"]
        .as_str()
        .unwrap_or("未知日期")
        .to_string();

    let tags = frontmatter["tags"]
        .as_array()
        .unwrap_or(&toml::value::Array::new())
        .iter()
        .filter_map(|t| t.as_str().map(String::from))
        .collect();

    // 将Markdown转换为HTML
    let html_content = markdown::to_html(content);

    // 创建Post实例
    Some(Post {
        name: file_name.to_string(),
        title,
        create_date: date,
        tags,
        content: html_content,
    })
}

/// 分割文章的前置元数据和内容主体
fn split_frontmatter_and_content(content: &str) -> (&str, &str) {
    let mut parts = content.splitn(3, "---");
    let _ = parts.next(); // 跳过第一部分（应该是空的）
    let frontmatter = parts.next().unwrap_or("");
    let content = parts.next().unwrap_or("");

    (frontmatter, content)
}

/// 准备输出目录结构
fn prepare_output_directories() -> io::Result<()> {
    create_dir_all("dist/posts")?;
    Ok(())
}

/// 初始化Tera模板引擎
fn init_template_engine() -> Result<Tera, Box<dyn std::error::Error>> {
    match Tera::new("site/templates/**/*") {
        Ok(tera) => Ok(tera),
        Err(e) => {
            eprintln!("模板解析错误: {}", e);
            Err(e.into())
        }
    }
}

/// 生成首页
fn generate_index_page(tera: &Tera, posts: &[Post]) -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::new();
    context.insert("posts", posts);

    let rendered = tera.render("index.html", &context)?;
    let mut file = File::create("dist/index.html")?;
    file.write_all(rendered.as_bytes())?;

    Ok(())
}

/// 生成所有文章页面
fn generate_post_pages(tera: &Tera, posts: &[Post]) -> Result<(), Box<dyn std::error::Error>> {
    for post in posts {
        let mut context = Context::new();
        context.insert("title", &post.title);
        context.insert("content", &post.content);
        context.insert("create_date", &post.create_date);
        context.insert("tags", &post.tags);

        let rendered = tera.render("post.html", &context)?;
        let post_path = format!("dist/posts/{}.html", post.name);
        let mut file = File::create(post_path)?;
        file.write_all(rendered.as_bytes())?;
    }

    Ok(())
}

fn copy_static_files() -> io::Result<()> {
    let static_dir = Path::new("site/static");

    // 如果static目录不存在，创建一个空的静态目录
    if !static_dir.exists() {
        return create_dir_all("dist/static");
    }

    copy_dir_all(static_dir, "dist/static")
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !src.exists() {
        return Ok(());
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
