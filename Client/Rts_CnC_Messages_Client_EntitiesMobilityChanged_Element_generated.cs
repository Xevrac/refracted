using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntitiesMobilityChanged_Element
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntitiesMobilityChanged.Element); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntitiesMobilityChanged.Element)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize CanMove
            s.Write(value.CanMove);
            //  Serialize CanTurn
            s.Write(value.CanTurn);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            Rts.CnC.Messages.Client.EntitiesMobilityChanged.Element value = default(Rts.CnC.Messages.Client.EntitiesMobilityChanged.Element);
            DeserializeValue(s, ref value);
            return value;
        }
        
        public static void DeserializeValue(System.IO.Stream s, ref Rts.CnC.Messages.Client.EntitiesMobilityChanged.Element value)
        {
            var valueRef = __makeref(value);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize CanMove
            s.Read(out value.CanMove);
            //  Deserialize CanTurn
            s.Read(out value.CanTurn);

        }
    }
}
