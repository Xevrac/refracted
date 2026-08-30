using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_FirestormActivated
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.FirestormActivated); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.FirestormActivated)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize Id
            s.Write(value.Id);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Activate
            s.Write(value.Activate);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.FirestormActivated)) as Rts.CnC.Messages.Client.FirestormActivated;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize Id
            s.Read(out value.Id);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Activate
            s.Read(out value.Activate);

            return value;
        }
        
    }
}
