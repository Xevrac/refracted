using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_CameraExtentsChanged
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.CameraExtentsChanged); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.CameraExtentsChanged)obj;
            //  Serialize XMinimum
            s.Write(value.XMinimum);
            //  Serialize XMaximum
            s.Write(value.XMaximum);
            //  Serialize ZMinimum
            s.Write(value.ZMinimum);
            //  Serialize ZMaximum
            s.Write(value.ZMaximum);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.CameraExtentsChanged)) as Rts.CnC.Messages.Client.CameraExtentsChanged;
            //  Deserialize XMinimum
            s.Read(out value.XMinimum);
            //  Deserialize XMaximum
            s.Read(out value.XMaximum);
            //  Deserialize ZMinimum
            s.Read(out value.ZMinimum);
            //  Deserialize ZMaximum
            s.Read(out value.ZMaximum);

            return value;
        }
        
    }
}
