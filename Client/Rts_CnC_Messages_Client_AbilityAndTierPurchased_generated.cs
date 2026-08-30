using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AbilityAndTierPurchased
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AbilityAndTierPurchased); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AbilityAndTierPurchased)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize ActiveAbilityId
            s.Write(value.ActiveAbilityId);
            //  Serialize PassiveAbilityId
            s.Write(value.PassiveAbilityId);
            //  Serialize SkillTier
            s.Write(value.SkillTier);
            //  Serialize MillisecondsToReenable
            s.Write(value.MillisecondsToReenable);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AbilityAndTierPurchased)) as Rts.CnC.Messages.Client.AbilityAndTierPurchased;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize ActiveAbilityId
            s.Read(out value.ActiveAbilityId);
            //  Deserialize PassiveAbilityId
            s.Read(out value.PassiveAbilityId);
            //  Deserialize SkillTier
            s.Read(out value.SkillTier);
            //  Deserialize MillisecondsToReenable
            s.Read(out value.MillisecondsToReenable);

            return value;
        }
        
    }
}
