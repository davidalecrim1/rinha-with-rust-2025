# v0.0.1

**Commit:**: d348b44a5b26355e5cb3fbd83d5d0cc165599e5b
**Load Test Commit Version**: 227406565b4524e718e3bcc062ee8a9615fec15a
**Load Test Result**: report_20250815_193412.html

**Changes**:
- This version uses an API to save payments in a redis list as queue;
- A worker pool consumes the queue and process the payments in the default endpoint;
- A redis is used with a sorted set to see the stored payments;
- This version has inconsistencies and will be improved.
