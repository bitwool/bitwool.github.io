import os, strformat, strutils, std/re
import markdown
import nimja

type
  Post = object
    path: string
    href: string
    title: string
    date: string
    tags: seq[string]
    content: string

proc `$`(p: Post): string =
  fmt"""Post(
    path: {p.path}
    href: {p.href}
    title: {p.title}
    date: {p.date}
    tags: {p.tags}
    content: {p.content}
  )"""


# 把markdown分成两部分，metadata和content
# metadata使用yaml解析出title、date、tags等字段
# content使用markdown解析为html
proc split_post(filepath: string): (string, string) =
  let fileContent = readFile(filepath)
  let re = re"(?s)^\+\+\+\n(.*?)\n\+\+\+\n(.*)"
  var matches: array[2, string]
  if match(fileContent, re, matches):
    (matches[0], matches[1])
  else:
    ("", fileContent)

proc parse_metadata(metadata: string):(string, string, seq[string]) = 
  let lines = metadata.split('\n')
  var title = ""
  var date = ""
  var tags = @[""]
  for line in lines:
    let parts = line.split(':')
    case parts[0].strip():
    of "title":
      title = parts[1].strip()
    of "date":
      date = parts[1].strip()
    of "tags":
      tags = parts[1].strip().split(',')
  (title, date, tags)

proc mdPathToHtmlPath(mdPath: string): string =
  let (dir, name, ext) = splitFile(mdPath)
  let newDir = dir.replace("site", "dist")
  result = newDir / name & ".html"

proc parse_posts(): seq[Post] = 
  result = @[]
  try:
    for file in walkDir("./site/posts"):
      let path = file.path
      let htmlPath = mdPathToHtmlPath(path)
      try:
        let (metadata, content) = split_post(path)
        let (title, date, tags) = parse_metadata(metadata)
        var post = Post(
          path: htmlPath,
          href: htmlpath[5 .. ^1],
          title: title,
          date: date,
          tags: tags,
          content: markdown(content)
        )

        result.add(post)
      except IOError:
        echo fmt"无法读取文件 {path}: {getCurrentExceptionMsg()}"
      except:
        echo fmt"处理文件 {path} 时发生错误: {getCurrentExceptionMsg()}"
  except OSError:
    echo "无法访问posts目录: " & getCurrentExceptionMsg()

proc renderPost(post: Post): string =
  compileTemplateFile("./site/templates/post.nimja", baseDir = getScriptDir() / ".." )

proc renderIndex(posts: seq[Post]): string =
  compileTemplateFile("./site/templates/index.nimja", baseDir = getScriptDir() / ".." )

proc saveHtml(path: string, content: string) =
  createDir(parentDir(path))
  writeFile(path, content)

proc cpStatic() =
  let sourceDir = "./site/static"
  let targetDir = "./dist/static"
  copyDir(sourceDir, targetDir)
  

when isMainModule:
  let posts = parse_posts()
  for post in posts:
    saveHtml(post.path, renderPost(post))
  saveHtml("./dist/index.html", renderIndex(posts))
  cpStatic()