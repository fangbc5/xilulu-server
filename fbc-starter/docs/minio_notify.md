为minio绑定上传通知，向kafka发送消息
1、设置别名以及用户名和密码
mc alias set myminio http://localhost:9000 rustfsadmin rustfsadmin
2、使用新的用户名和密码重新登录，并创建桶public
3、配置kafka监听
mc admin config set myminio notify_kafka:1 \
  brokers="kafka:9092" \
  topic="minio-events" \
  username="" \
  password="" \
  tls="off"
4、重启minio
mc admin service restart myminio
5、为public桶绑定事件
mc event add myminio/public \
  arn:minio:sqs::1:kafka \
  --event put