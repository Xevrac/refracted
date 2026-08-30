using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_PlayerAbilityExclusionZoneCreated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.PlayerAbilityExclusionZoneCreated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.PlayerAbilityExclusionZoneCreated)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Radius
            s.Write(value.Radius);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.PlayerAbilityExclusionZoneCreated)) as Rts.CnC.Messages.Client.PlayerAbilityExclusionZoneCreated;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Radius
            s.Read(out value.Radius);

            return value;
        }
        
    }
}
