/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../../..";
import * as Rivet from "../../../../../../../../api";
import * as core from "../../../../../../../../core";
export declare const CreateGameBuildRequest: core.serialization.ObjectSchema<serializers.cloud.games.CreateGameBuildRequest.Raw, Rivet.cloud.games.CreateGameBuildRequest>;
export declare namespace CreateGameBuildRequest {
    interface Raw {
        display_name: serializers.DisplayName.Raw;
        image_tag: string;
        image_file: serializers.upload.PrepareFile.Raw;
        multipart_upload?: boolean | null;
        kind?: serializers.cloud.games.BuildKind.Raw | null;
        compression?: serializers.cloud.games.BuildCompression.Raw | null;
    }
}
