# electron_updater

electron 增量更新仓库

* electron应该在可以安全退出的情况下调用更新程序，并在调用后退出electron程序，防止更新出错；
* 更新程序会尝试结束electron程序，如果没有结束掉electron程序将不会继续运行；
* 如果更新程序没有安装，则它将被下载到用户的临时文件夹中，并在更新完成后被删除；todo
