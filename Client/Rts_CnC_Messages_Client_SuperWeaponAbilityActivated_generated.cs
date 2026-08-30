using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_SuperWeaponAbilityActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.SuperWeaponAbilityActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.SuperWeaponAbilityActivated)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize AbilityId
            s.Write(value.AbilityId);
            //  Serialize PercentAtActivation
            s.Write(value.PercentAtActivation);
            //  Serialize TargetLocation
            s.Write(value.TargetLocation);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.SuperWeaponAbilityActivated)) as Rts.CnC.Messages.Client.SuperWeaponAbilityActivated;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize AbilityId
            s.Read(out value.AbilityId);
            //  Deserialize PercentAtActivation
            s.Read(out value.PercentAtActivation);
            //  Deserialize TargetLocation
            s.Read(out value.TargetLocation);

            return value;
        }
        
    }
}
