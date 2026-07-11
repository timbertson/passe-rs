#!/usr/bin/env bash
set -eux

# To be run from root directory

export GOOGLE_APPLICATION_CREDENTIALS="$(pwd)/deploy/secrets/passe.json"
REGION="australia-southeast2"
ARTIFACT_REGISTRY="australia-southeast2-docker.pkg.dev"
PROJECT_ID=passe-225909
IMAGE="$ARTIFACT_REGISTRY/$PROJECT_ID/passe/app"

if [ "${1:-}" != "--quick" ]; then
	gup -u ui/all
	docker-credential-gcr configure-docker --registries="$ARTIFACT_REGISTRY"
	docker build . -f deploy/Dockerfile -t "$IMAGE"
	docker push "$IMAGE"
fi
SECRET_TOKEN_FILE="deploy/secrets/token"
gup -u "$SECRET_TOKEN_FILE"
gcloud \
	--access-token-file="$SECRET_TOKEN_FILE" \
	--project="$PROJECT_ID" \
	run deploy passe \
	--image="$IMAGE" \
	--max=1 \
	--min=0 \
	--scaling=auto \
	--no-cpu-boost \
	--cpu-throttling \
	--memory=512Mi \
	--region="$REGION" \
	--allow-unauthenticated \
	--add-volume="type=cloud-storage,mount-path=/var/passe,bucket=passe-rs-storage" \
	--port=8080 \
;
