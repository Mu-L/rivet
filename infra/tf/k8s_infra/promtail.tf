# locals {
# 	labels = {
# 		ns = var.namespace
# 	}
# 	external_labels = join(",", [
# 		for key, value in local.labels:
# 		"${key}=${value}"
# 	])
# }

# resource "kubernetes_namespace" "promtail" {
# 	metadata {
# 		name = "promtail"
# 	}
# }

# resource "helm_release" "promtail" {
# 	name = "promtail"
# 	namespace = kubernetes_namespace.promtail.metadata.0.name
# 	repository = "https://grafana.github.io/helm-charts"
# 	chart = "promtail"
# 	version = "6.15.1"
# 	values = [yamlencode({
# 		config = {
# 			clients = [{
# 				url = "http://loki.loki.svc.cluster.local:3100/loki/api/v1/push"
# 				tenant_id = 1
# 			}]
# 			snippets = {
# 				pipelineStages = [
# 					{
# 						cri: {}
# 					},
# 					# # Limit logs to only the "rivet-service" k8s namespace
# 					# {
# 					# 	match: {
# 					# 		pipeline_name = "drop-all"
# 					# 		selector = "{namespace!=\"rivet-service\"}"
# 					# 		action = "drop"
# 					# 	}
# 					# }
# 				]
# 				# NOTE: Do not add relabel configs to `common` below, add them here
# 				extraRelabelConfigs = [
# 					{
# 						action = "replace"
# 						source_labels = ["__meta_kubernetes_pod_node_name"]
# 						target_label = "node"
# 					},
# 					{
# 						action = "replace"
# 						source_labels = ["__meta_kubernetes_pod_uid"]
# 						target_label = "alloc"
# 					},
# 					{
# 						action = "replace"
# 						# Label names cannot use dots or slashes, convert to underscore
# 						# https://groups.google.com/g/prometheus-users/c/dGx7MArW-eE
# 						# https://github.com/prometheus/docs/issues/735
# 						source_labels = ["__meta_kubernetes_pod_label_app_kubernetes_io_name"]
# 						target_label = "service"
# 					},
# 					# Doesn't work for some reason
# 					{
# 						action = "labeldrop"
# 						regex = "^(host|filename)$"
# 					}
# 				]

# 				# Remove default relabel config (see helm chart values.yaml)
# 				common = [
# 					# {
# 					# 	action = "replace"
# 					# 	source_labels = ["__meta_kubernetes_namespace"]
# 					# 	target_label = "namespace"
# 					# },
# 					# {
# 					# 	action = "replace"
# 					# 	replacement = "$1"
# 					# 	separator = "/"
# 					# 	source_labels = ["namespace", "app"]
# 					# 	target_label = "job"
# 					# },
# 					# {
# 					# 	action = "replace"
# 					# 	source_labels = ["__meta_kubernetes_pod_container_name"]
# 					# 	target_label = "container"
# 					# },
# 					{
# 						action = "replace"
# 						replacement = "/var/log/pods/*$1/*.log"
# 						separator = "/"
# 						source_labels = ["__meta_kubernetes_pod_uid", "__meta_kubernetes_pod_container_name"]
# 						target_label = "__path__"
# 					},
# 					{
# 						action = "replace"
# 						replacement = "/var/log/pods/*$1/*.log"
# 						regex = true
# 						separator = "/"
# 						source_labels = ["__meta_kubernetes_pod_annotationpresent_kubernetes_io_config_hash", "__meta_kubernetes_pod_annotation_kubernetes_io_config_hash", "__meta_kubernetes_pod_container_name"]
# 						target_label = "__path__"
# 					}
# 				]
# 				# Remove default scrape config (see helm chart values.yaml)
# 				scrapeConfigs = <<-EOF
# 					# See also https://github.com/grafana/loki/blob/master/production/ksonnet/promtail/scrape_config.libsonnet for reference
# 					- job_name: kubernetes-pods
# 					  pipeline_stages:
# 					    {{- toYaml .Values.config.snippets.pipelineStages | nindent 4 }}
# 					  # Limits logs to only the "rivet-service" k8s namespace
# 					  kubernetes_sd_configs:
# 					    - role: pod
# 					      namespaces:
# 					        names:
# 					          - rivet-service
# 					  relabel_configs:
# 					    {{- if .Values.config.snippets.addScrapeJobLabel }}
# 					    - replacement: kubernetes-pods
# 					      target_label: scrape_job
# 					    {{- end }}
# 					    {{- toYaml .Values.config.snippets.common | nindent 4 }}
# 					    {{- with .Values.config.snippets.extraRelabelConfigs }}
# 					    {{- toYaml . | nindent 4 }}
# 					    {{- end }}
# 					EOF
# 			}
# 		}

# 		extraArgs = [
# 			"-client.external-labels=${local.external_labels}"
# 		]
# 	})]
# }