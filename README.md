# homelabd

`homelabd` is a daemon to run on nodes in my homelab. It's intended to allow
each node to regularly broadcast information about itself, allowing for
complex tasks like service discovery and trivial tasks like building a
leaderboard for host uptime.

`homelabd` uses multicast to broadcast messages and, in some cases, unicast
HTTP to retrieve content directly from a node. Eventually I'd like to see
more comprehensive clustering features such as a distributed KV store.
