---
title= "使用nim编写静态博客生成器"
date= "2025-01-08"
tags= ["nim"]
---

2024 年一直在折腾不同的语言，尝试了很多种最终都半途而废。在折腾过程中了解到了 Nim 语言。Nim 官网对自己的定位是高效、优雅、复有表现力。经过初步学习后，使用 Nim 把自己工作中的一个小工具重写了一遍，写起来还是很舒服。写完之后，不知道写啥好，干脆写个博客吧。一个是提升 Nim 的熟练度，另外一个以为契机开始写博客，提升内容输出的能力。毕竟搭建从未停止，写作从未开始。

## 设计思路

第一步只实现最简单的功能

- 扫描博客文档目录，把 markdown 文件转成静态 html 页面，并且解析博客元信息
- 汇总博客元信息以及内容 html，填充到定义好的模板文件中

## 实现过程

### 安装依赖

该项目中主要用到两个核心依赖

- markdown 把 markdown 文件转换为 html
- nimja Nim 语言中一个类似 jinja 的模板引擎

```shell
nimble install markdown
nimble install nimja
```

ngen.nimble
```nim
requires "markdown"
requires "nimja"
```

### markdown文档解析

Nim 的 markdown 库不支持对元信息的解析，这里采用的方法是：
1. 使用正则表达式把markdown文件分为两部分，元信息使用`+++`包围的部分，剩下的markdown部分作为博客的正文
2. 元信息使用简单的键值对来表示，对于tags这种需要支持多值的类型，使用逗号拼接
3. 正文部分则比较简单，直接使用markdown解析为html即可

首先是对文章的结构定义
```nim
type
  Post = object
    path: string
    href: string
    title: string
    date: string
    tags: seq[string]
    content: string
```

markdown拆分
```nim
proc split_post(filepath: string): (string, string) =
  let fileContent = readFile(filepath)
  let re = re"(?s)^\+\+\+\n(.*?)\n\+\+\+\n(.*)"
  var matches: array[2, string]
  if match(fileContent, re, matches):
    (matches[0], matches[1])
  else:
    ("", fileContent)
```

元信息解析
```nim
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
```


markdown解析直接调用`markdown()`即可，不需要单独封装函数，合并到posts解析部分
```nim
type
  Post = object
    path: string
    href: string
    title: string
    date: string
    tags: seq[string]
    content: string

proc parse_posts(): seq[Post] = 
  result = @[]
  for file in walkDir("./site/posts"):
    let path = file.path
    let htmlPath = mdPathToHtmlPath(path)
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
```


### 模板渲染
解析之后得到了`posts`数组，接下来把解析到的结果渲染到模板，再保存为html即可。按照最简功能设计，只需要主页index和文章页post的模板

主页模板
```html
<body>
    {% for post in posts %}
    <a href="{{post.href}}"> {{post.title}}</a>
    {% endfor %}
</body>
```
文章模板
```html

<body>
    <h1>{{post.title}}</h1>
    <p>{{post.date}} </p>

    {% for tag in post.tags %}
    {{tag}} 
    {% endfor %}

    {{post.content}}
</body>
```

模板渲染使用`nimja`库中的`compileTemplateFile`,这是一个宏，可以对模板文件编译，并且注入变量
```nim
proc renderPost(post: Post): string =
  compileTemplateFile("./site/templates/post.nimja", baseDir = getScriptDir() / ".." )

proc renderIndex(posts: seq[Post]): string =
  compileTemplateFile("./site/templates/index.nimja", baseDir = getScriptDir() / ".." )
```

### 其他
- 文章忽略了保存为html文件以及static资源复制的部分，这些逻辑比较简单，不在文件中列出
- 关于博客样式，借助大模型和bootstrap生成了一个勉强能看的版本，后续再做优化
- 代码高亮用到[highlightjs](https://highlightjs.org/)库，配色方案使用[catppuccin/highlightjs](https://github.com/catppuccin/highlightjs)中的`Latte`

## 部署
部署选择了Github Pages方案，通过Github Action实现自动部署
```yaml
name: Deploy site to Pages

on:
  push:
    branches:
      - master

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - name: Checkout
      uses: actions/checkout@v4
    - name: Install libpcre
      run: sudo apt-get install libpcre3-dev libpcre3
    - name: Setup Nim
      uses: jiro4989/setup-nim-action@v2
      with:
        # nim-version: '2.0.0'
        repo-token: ${{ secrets.GH_PAT }}
    - name: Nimble Run
      run: nimble run
    - name: Deploy
      uses: peaceiris/actions-gh-pages@v4
      with:
        github_token: ${{ secrets.GH_PAT }}
        publish_dir: ./dist
```


## 总结
以上实现的功能很简单，nim写起来也很舒服，后续会继续对这个生成器进行优化，完成功能。对于一些实现相对负责的功能，也会继续写博客来分享。