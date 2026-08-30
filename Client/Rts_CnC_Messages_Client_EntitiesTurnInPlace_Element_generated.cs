using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntitiesTurnInPlace_Element
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntitiesTurnInPlace.Element); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntitiesTurnInPlace.Element)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Facing
            s.Write(value.Facing);
            //  Serialize TimeDuration
            s.Write(value.TimeDuration);
            //  Serialize AngularVelocity
            s.Write(value.AngularVelocity);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            Rts.CnC.Messages.Client.EntitiesTurnInPlace.Element value = default(Rts.CnC.Messages.Client.EntitiesTurnInPlace.Element);
            DeserializeValue(s, ref value);
            return value;
        }
        
        public static void DeserializeValue(System.IO.Stream s, ref Rts.CnC.Messages.Client.EntitiesTurnInPlace.Element value)
        {
            var valueRef = __makeref(value);
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Facing
            s.Read(out value.Facing);
            //  Deserialize TimeDuration
            s.Read(out value.TimeDuration);
            //  Deserialize AngularVelocity
            s.Read(out value.AngularVelocity);

        }
    }
}
