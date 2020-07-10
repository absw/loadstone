pipeline {
    agent {
        docker { image 'rustembedded/cross:thumbv7em-none-eabihf-0.2.1' }
    }
    stages {
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
        stage('Test') {
            steps {
                sh 'cargo test'
            }

        }
        stage('Build') {
            steps {
                sh 'export CROSS_DOCKER_IN_DOCKER=true'
                sh 'cross build --target thumbv7em-none-eabih'
            }
        }
    }
}
