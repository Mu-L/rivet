/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common, group } from "../../../../index";
export declare const Summary: core.serialization.ObjectSchema<serializers.group.Summary.Raw, Rivet.group.Summary>;
export declare namespace Summary {
    interface Raw {
        group_id: string;
        display_name: common.DisplayName.Raw;
        avatar_url?: string | null;
        external: group.ExternalLinks.Raw;
        is_developer: boolean;
        bio: common.Bio.Raw;
        is_current_identity_member: boolean;
        publicity: group.Publicity.Raw;
        member_count: number;
        owner_identity_id: string;
    }
}
