# Message Bus
People (like me) like to ride buses. They take you places. You know where they're going to go. And you don't have to go underground so you can still doom-scroll while riding them! Computers provide [buses](https://en.wikipedia.org/wiki/Bus_(computing)) too, just they're moving data not people (yet).


```bash
echo -ne 'MAKE test_topic\r\n' | netcat localhost 8080
```

# Resources 
- https://www.ibm.com/cloud/learn/message-brokers
- https://en.wikipedia.org/wiki/Message_broker
- https://books.google.com/books?id=2EonCgAAQBAJ&pg=PA71#v=onepage&q&f=false
- https://github.com/rabbitmq/internals/
- https://docs.microsoft.com/en-us/azure/service-bus-messaging/service-bus-messaging-overview
- https://hackernoon.com/publish-subscribe-design-pattern-introduction-to-scalable-messaging-781k3tae
- https://kafka.apache.org/0100/protocol.html
- https://news.ycombinator.com/item?id=19016466

## Topics
- ordering + exactly once delivery
- sharding / partitioning
- leader election
- dead letter queues, message retries
- authentication, producer verification, security
- consumer groups

## Systems
- Kinesis, SQS
- RabbitMQ
- Kafka
- NATS