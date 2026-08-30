using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityMobilityChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityMobilityChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityMobilityChanged)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize CanMove
            s.Write(value.CanMove);
            //  Serialize CanTurn
            s.Write(value.CanTurn);
            //  Serialize TimeStamp
            s.Write(value.TimeStamp);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityMobilityChanged)) as Rts.CnC.Messages.Client.EntityMobilityChanged;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize CanMove
            s.Read(out value.CanMove);
            //  Deserialize CanTurn
            s.Read(out value.CanTurn);
            //  Deserialize TimeStamp
            s.Read(out value.TimeStamp);

            return value;
        }
        
    }
}
