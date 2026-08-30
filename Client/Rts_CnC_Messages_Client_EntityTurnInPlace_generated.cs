using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityTurnInPlace
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityTurnInPlace); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityTurnInPlace)obj;
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
            //  Serialize TimeStamp
            s.Write(value.TimeStamp);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityTurnInPlace)) as Rts.CnC.Messages.Client.EntityTurnInPlace;
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
            //  Deserialize TimeStamp
            s.Read(out value.TimeStamp);

            return value;
        }
        
    }
}
