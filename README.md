### charon
目前实现的特性如下：
1. 支持的类型：bool、long、double、string、function、class
2. 函数可以赋值给变量、类的字段、作为参数或者返回值（method跟function不同，不是first-class类型，不能赋值给变量，类字段，也不能作为函数、方法的参数或者返回值）
3. 支持常见的语言结构，比如：if-elseif-else，while-break-continue
4. 支持定义类、方法，方法中可以通过'this'访问当前实例的字段
5. 支持简单的`ffi`机制，用于实现`charon`做不到的事情，比如打印输出: __print, __println


构建：
1. 需要先安装rust，如果之前未安装过，可以先按rust官网的指示去安装一下
2. clone一下代码仓库
3. cd 到 charon 项目根目录
4. 执行：cargo build --release

执行完上面的步骤后，会在`target/release`目录下生成3个可执行程序：
1. `charonc`: charon 的编译器，会将代码编译成字节码，文件后缀：`.charonbc`，字节码格式类似于Java字节码
2. `charonp`: 反汇编`charonc`生成的字节码文件`.charonbc`，功能类似于javap
3. `charon`: 虚拟机可执行程序，可以传入`charon`源代码，也可以传入`charonc`编译生成的字节码

#### 代码示例
工程根目录下有个`examples`，里面有一些示例代码可以参考。

下面用`charon`构建一棵二叉树，并进行中序遍历作为一个例子：

```
#!/usr/bin/env charon

// 调用createBinaryTree创建一棵二叉树，然后通过inOrder进行中序遍历
inOrder(createBinaryTree());

class Node {}

//              10
//            /    \
//           6      14
//         /  \    /  \
//        4    8  12  16
//
func createBinaryTree() {
    var left = Node();
    left.value = 4;

    var right = Node();
    right.value = 8;

    var l = Node();
    l.value = 6;
    l.left = left;
    l.right = right;

    var left = Node();
    left.value = 12;

    var right = Node();
    right.value = 16;

    var r = Node();
    r.value = 14;
    r.left = left;
    r.right = right;

    var root = Node();
    root.value = 10;
    root.left = l;
    root.right = r;

    return root;
}

func inOrder(root) {
    if (root.left) {
        inOrder(root.left);
    }
    __println(root.value);
    if (root.right) {
        inOrder(root.right);
    }
}
```

上面的代码示例可以有如下几种方法执行：
1. 直接调用虚拟机执行:
    * `charon binary-tree.charon`

2. 先编译成字节码，再交由虚拟机执行
    * `charonc binary-tree.charon`
    * `charon binary-tree.charonbc`

3. `charon`是支持`shebang`，因此可以给源码文件加上可执行权限，然后直接执行
    * `chmod +x binary-tree.charon`
    * `./binary-tree.charon`


另外从上面的代码可以看到：
1. 函数不必提前声明即可使用
2. 变量可以重定义
3. 跟很多动态语言一样类的字段不必在类体中定义
4. 对于未赋值的类字段，读取时将得到null
5. 像null之类的值在作为bool表达式时会隐式转换为false
6. ...

#### 后续计划
* 后面计划整理几篇相关的文档以期加深对编译器&虚拟机相关技术的理解，预计会补充以下几篇文章：
    1. 词法介绍&词法分析器的实现
    2. 语法介绍&语法分析器的实现
    3. 语意分析、字节码设计&字节码生成（charon是基于栈的，也会和基于寄存器的实现简单对比下）
    4. 虚拟机的实现（比如栈帧的布局，对于method 'this'的支持实现和java不太相同，以及ffi的支持，后面会简单讨论一下）
    5. 编译器&虚拟机中的错误处理（编译过程中的行号、列号信息处理、关于错误恢复的讨论，运行时的栈溢出检测等）


受制于能力和假期时间限制，原本想支持的一些特性，比如类的自定义构造函数等等没来得及搞，原本想支持一个简单的gc的，也没来得及做，并且测试也很少😂，若有大佬发现bug或者有建议，希望帮忙指正～
