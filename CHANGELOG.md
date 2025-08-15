# v0.0.1

**Commit:**:
**Load Test Commit Version**:
**Load Test Result**:

**Changes**:
- This version uses an API to save payments in a redis list as queue;
- A worker pool consumes the queue and process the payments in the default endpoint;
- A redis is used with a sorted set to see the stored payments;
- This version has inconsistencies and will be improved.
