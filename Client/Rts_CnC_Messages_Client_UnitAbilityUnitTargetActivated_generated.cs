using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UnitAbilityUnitTargetActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UnitAbilityUnitTargetActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UnitAbilityUnitTargetActivated)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize TargetPlayerId
            s.Write(value.TargetPlayerId);
            //  Serialize TargetEntityId
            s.Write(value.TargetEntityId);
            //  Serialize UnitAbility
            s.Write(value.UnitAbility);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UnitAbilityUnitTargetActivated)) as Rts.CnC.Messages.Client.UnitAbilityUnitTargetActivated;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize TargetPlayerId
            s.Read(out value.TargetPlayerId);
            //  Deserialize TargetEntityId
            s.Read(out value.TargetEntityId);
            //  Deserialize UnitAbility
            s.Read(out value.UnitAbility);

            return value;
        }
        
    }
}
