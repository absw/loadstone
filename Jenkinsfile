#!groovy

podTemplate(
    yaml: """
apiVersion: v1
kind: Pod
metadata:
  labels:
spec:
  containers:
  - name: cross
    image: rustembedded/cross:thumbv7em-none-eabihf-0.2.1
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
        container('cross') {
            stage('Test') {
                sh 'cargo test'
            }
            stage('Build') {
                sh 'export CROSS_DOCKER_IN_DOCKER=true'
                sh 'cross build --target thumbv7em-none-eabihf'
            }
        }
    }
}