using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_UnitAbilityGroundTargetStarted
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.UnitAbilityGroundTargetStarted); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.UnitAbilityGroundTargetStarted)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize UnitAbility
            s.Write(value.UnitAbility);
            //  Serialize TargetPosition
            s.Write(value.TargetPosition);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.UnitAbilityGroundTargetStarted)) as Rts.CnC.Messages.Client.UnitAbilityGroundTargetStarted;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize UnitAbility
            s.Read(out value.UnitAbility);
            //  Deserialize TargetPosition
            s.Read(out value.TargetPosition);

            return value;
        }
        
    }
}
