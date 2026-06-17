sed -i 's/inner: Arc<crate::testing::TestOrchestrator>/inner: Arc<tokio::sync::Mutex<crate::testing::TestOrchestrator>>/g' crates/arkhe-kernel/src/lib.rs

sed -i 's/Ok(Self { inner: Arc::new(crate::testing::TestOrchestrator::new(/Ok(Self { inner: Arc::new(tokio::sync::Mutex::new(crate::testing::TestOrchestrator::new(/g' crates/arkhe-kernel/src/lib.rs

sed -i 's/Arc::new(crate::testing::deps::Ed25519Signer::new_random()))), rt:/Arc::new(crate::testing::deps::Ed25519Signer::new_random())))), rt:/g' crates/arkhe-kernel/src/lib.rs

sed -i 's/self.inner.register_test_agent/self.rt.block_on(async { self.inner.lock().await.register_test_agent/g' crates/arkhe-kernel/src/lib.rs

sed -i 's/_max_samples))); Ok(())/_max_samples))); }); Ok(())/g' crates/arkhe-kernel/src/lib.rs
sed -i 's/_concurrency))); Ok(())/_concurrency))); }); Ok(())/g' crates/arkhe-kernel/src/lib.rs
sed -i 's/_kill_percentage))); Ok(())/_kill_percentage))); }); Ok(())/g' crates/arkhe-kernel/src/lib.rs
sed -i 's/SecurityTestAgent::new()); Ok(())/SecurityTestAgent::new())); }); Ok(())/g' crates/arkhe-kernel/src/lib.rs
sed -i 's/_required_policies))); Ok(())/_required_policies))); }); Ok(())/g' crates/arkhe-kernel/src/lib.rs
sed -i 's/_test_count))); Ok(())/_test_count))); }); Ok(())/g' crates/arkhe-kernel/src/lib.rs

sed -i 's/self.inner.run_all_tests()/self.inner.lock().await.run_all_tests()/g' crates/arkhe-kernel/src/lib.rs
sed -i 's/self.inner.stats()/self.inner.lock().await.stats()/g' crates/arkhe-kernel/src/lib.rs

