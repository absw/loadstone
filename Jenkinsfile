pipeline {
    agent {
        docker {
            image 'rustembedded/cross:thumbv7em-none-eabihf-0.2.1'
            label 'secure_bootloader_builder'
        }
    }
    stages {
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
