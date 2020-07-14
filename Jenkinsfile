#!groovy

podTemplate(
    yaml: """
apiVersion: v1
kind: Pod
metadata:
  labels:
spec:
  containers:
  - name: rust
    image: rustlang/rust:nightly-stretch
    command:
    - cat
    tty: true
    resources:
      limits:
        cpu: 1.5
        memory: 2Gi
      requests:
        cpu: 300m
        memory: 512Mi
"""
    ) {
    node(POD_LABEL) {
        stage('Checkout SCM') {
            checkout([
                $class: "GitSCM",
                branches: scm.branches,
                extensions: scm.extensions + [
                    [$class: "GitLFSPull"]
                ],
                userRemoteConfigs: scm.userRemoteConfigs
            ])
        }
        container('rust') {
            stage('Test') {
                sh 'cargo test'
            }
            stage('Build') {
                sh 'rustup target add thumbv7em-none-eabihf'
                sh './cargo_emb check'
            }
        }
    }
}
